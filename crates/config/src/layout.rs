use crate::{
    model::{direct, overlaps, EDGES},
    *,
};

const EMPTY_RANGE_WARNING: &str = "Layout overlap rounds to an empty integer percentage range; link omitted";

/// Derive links from facing edges in native screen points. See LAYOUT.md.
/// Invalid geometry is ignored here; Config::validate rejects it.
pub fn derive_links(screens: &[ScreenCfg]) -> DerivedLinks {
    native_links(screens).0
}

pub(crate) fn warnings(screens: &[ScreenCfg]) -> Vec<String> {
    native_links(screens).1
}

fn native_links(screens: &[ScreenCfg]) -> (DerivedLinks, Vec<String>) {
    let mut links = Vec::new();
    let mut warnings = Vec::new();
    for source in screens {
        for edge in EDGES {
            if explicit_links(source, edge, &mut links) {
                continue;
            }
            let Some(a) = source.rect.filter(valid_rect) else {
                continue;
            };
            for target in screens {
                if std::ptr::eq(source, target) {
                    continue;
                }
                let Some(b) = target.rect.filter(valid_rect) else {
                    continue;
                };
                // Origin order also prevents a sub-point-wide screen from linking backwards
                // merely because both of its opposite edges fit inside the 1-point tolerance.
                let touching = match edge {
                    Edge::Left => b.x < a.x && (a.x - (b.x + b.w)).abs() <= 1.0,
                    Edge::Right => b.x > a.x && (a.x + a.w - b.x).abs() <= 1.0,
                    Edge::Top => b.y < a.y && (a.y - (b.y + b.h)).abs() <= 1.0,
                    Edge::Bottom => b.y > a.y && (a.y + a.h - b.y).abs() <= 1.0,
                };
                if !touching {
                    continue;
                }
                let (ao, ad, bo, bd) = perpendicular_spans(a, b, edge);
                if (ao + ad).min(bo + bd) - ao.max(bo) <= 1.0 {
                    continue;
                }
                if let Some(link) = overlap_link(source, target, edge, a, b) {
                    links.push(link);
                } else {
                    warn_once(&mut warnings, EMPTY_RANGE_WARNING);
                }
            }
        }
    }
    (links, warnings)
}

/// Reproduce the service's editor geometry only for Synergy 3 db.json import.
/// Its asymmetric probes and later-identical-key overwrite rule are intentional.
pub(crate) fn derive_synergy3_links(screens: &[ScreenCfg]) -> (DerivedLinks, Vec<String>) {
    let mut links: DerivedLinks = Vec::new();
    let mut warnings = Vec::new();
    for source in screens {
        for edge in EDGES {
            if explicit_links(source, edge, &mut links) {
                continue;
            }
            let Some(a) = source.rect.and_then(expand) else {
                continue;
            };
            let mut probe = a;
            match edge {
                Edge::Left => probe.x -= 1.0,
                Edge::Right => probe.w += 1.0,
                Edge::Top => probe.y -= 1.0,
                Edge::Bottom => probe.h += 1.0,
            }
            for target in screens {
                if std::ptr::eq(source, target) {
                    continue;
                }
                let Some(b) = target.rect.and_then(expand) else {
                    continue;
                };
                if !overlaps(probe, b) {
                    continue;
                }
                if overlaps(a, b) {
                    warn_once(
                        &mut warnings,
                        "Expanded Synergy 3 editor rects overlap; imported service links can include extra directions",
                    );
                }
                let Some(link) = overlap_link(source, target, edge, a, b) else {
                    warn_once(&mut warnings, EMPTY_RANGE_WARNING);
                    continue;
                };
                if let Some(existing) = links
                    .iter_mut()
                    .find(|l| l.from == link.from && l.edge == edge && l.from_range == link.from_range)
                {
                    *existing = link;
                } else {
                    links.push(link);
                }
            }
        }
    }
    (links, warnings)
}

/// Return true even for a disconnected edge so geometry cannot fill it back in.
fn explicit_links(source: &ScreenCfg, edge: Edge, links: &mut DerivedLinks) -> bool {
    let Some(explicit) = &source.links else {
        return false;
    };
    let mut ranged = explicit.ranges.iter().filter(|l| l.edge == edge).peekable();
    if ranged.peek().is_some() {
        for link in ranged {
            links.push(DerivedLink {
                from: source.name.clone(),
                edge,
                from_range: link.from_range,
                to: link.to.clone(),
                to_range: link.to_range,
            });
        }
        return true;
    }
    if let Some(target) = direct(explicit, edge) {
        if !target.is_empty() {
            links.push(DerivedLink {
                from: source.name.clone(),
                edge,
                from_range: (0, 100),
                to: target.into(),
                to_range: (0, 100),
            });
        }
        return true;
    }
    false
}

fn perpendicular_spans(a: LayoutRect, b: LayoutRect, edge: Edge) -> (f64, f64, f64, f64) {
    match edge {
        Edge::Left | Edge::Right => (a.y, a.h, b.y, b.h),
        Edge::Top | Edge::Bottom => (a.x, a.w, b.x, b.w),
    }
}

fn overlap_link(
    source: &ScreenCfg,
    target: &ScreenCfg,
    edge: Edge,
    a: LayoutRect,
    b: LayoutRect,
) -> Option<DerivedLink> {
    let (ao, ad, bo, bd) = perpendicular_spans(a, b, edge);
    let start = ao.max(bo);
    let end = (ao + ad).min(bo + bd);
    if start >= end {
        return None;
    }
    let range = |o: f64, d: f64| {
        (
            (100.0 * (start - o) / d).round() as u8,
            (100.0 * (end - o) / d).round() as u8,
        )
    };
    let from_range = range(ao, ad);
    let to_range = range(bo, bd);
    if from_range.0 >= from_range.1 || to_range.0 >= to_range.1 {
        return None;
    }
    Some(DerivedLink {
        from: source.name.clone(),
        edge,
        from_range,
        to: target.name.clone(),
        to_range,
    })
}

fn valid_rect(rect: &LayoutRect) -> bool {
    [rect.x, rect.y, rect.w, rect.h, rect.x + rect.w, rect.y + rect.h]
        .iter()
        .all(|v| v.is_finite())
        && rect.w > 0.0
        && rect.h > 0.0
}

fn expand(rect: LayoutRect) -> Option<LayoutRect> {
    if !valid_rect(&rect) {
        return None;
    }
    let x = (rect.x - 0.5).max(0.0);
    let y = (rect.y - 0.5).max(0.0);
    let w = rect.x + rect.w + 0.5 - x;
    let h = rect.y + rect.h + 0.5 - y;
    if ![x, y, w, h].iter().all(|v| v.is_finite()) || w <= 0.0 || h <= 0.0 {
        return None;
    }
    Some(LayoutRect { x, y, w, h })
}

fn warn_once(warnings: &mut Vec<String>, message: &str) {
    if !warnings.iter().any(|warning| warning == message) {
        warnings.push(message.into());
    }
}

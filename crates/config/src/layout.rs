use crate::{
    model::{direct, overlaps, EDGES},
    *,
};

/// Exact service-converter geometry, including boundary clipping and probe asymmetry.
/// See LAYOUT.md. Invalid geometry is ignored here; Config::validate rejects it.
pub fn derive_links(screens: &[ScreenCfg]) -> DerivedLinks {
    let mut links: DerivedLinks = Vec::new();
    for source in screens {
        for edge in EDGES {
            if let Some(explicit) = &source.links {
                let ranged: Vec<_> = explicit.ranges.iter().filter(|l| l.edge == edge).collect();
                if !ranged.is_empty() {
                    for link in ranged {
                        links.push(DerivedLink {
                            from: source.name.clone(),
                            edge,
                            from_range: link.from_range,
                            to: link.to.clone(),
                            to_range: link.to_range,
                        });
                    }
                    continue;
                }
                if let Some(target) = direct(explicit, edge) {
                    // An empty string is an explicit disconnected edge.
                    if !target.is_empty() {
                        links.push(DerivedLink {
                            from: source.name.clone(),
                            edge,
                            from_range: (0, 100),
                            to: target.into(),
                            to_range: (0, 100),
                        });
                    }
                    continue;
                }
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
                let (ao, ad, bo, bd) = match edge {
                    Edge::Left | Edge::Right => (a.y, a.h, b.y, b.h),
                    Edge::Top | Edge::Bottom => (a.x, a.w, b.x, b.w),
                };
                let start = ao.max(bo);
                let end = (ao + ad).min(bo + bd);
                if start >= end {
                    continue;
                }
                let range = |o: f64, d: f64| {
                    (
                        ((100.0 * (start - o) / d).round()) as u8,
                        ((100.0 * (end - o) / d).round()) as u8,
                    )
                };
                let from_range = range(ao, ad);
                let to_range = range(bo, bd);
                // A sub-percent sliver cannot be represented by the frozen u8 contract.
                if from_range.0 >= from_range.1 || to_range.0 >= to_range.1 {
                    continue;
                }
                let link = DerivedLink {
                    from: source.name.clone(),
                    edge,
                    from_range,
                    to: target.name.clone(),
                    to_range,
                };
                if let Some(existing) = links
                    .iter_mut()
                    .find(|l| l.from == link.from && l.edge == edge && l.from_range == from_range)
                {
                    *existing = link;
                } else {
                    links.push(link);
                }
            }
        }
    }
    links
}

fn expand(rect: LayoutRect) -> Option<LayoutRect> {
    let x = (rect.x - 0.5).max(0.0);
    let y = (rect.y - 0.5).max(0.0);
    let w = rect.x + rect.w + 0.5 - x;
    let h = rect.y + rect.h + 0.5 - y;
    if ![x, y, w, h].iter().all(|v| v.is_finite()) || w <= 0.0 || h <= 0.0 {
        return None;
    }
    Some(LayoutRect { x, y, w, h })
}

/// Surface finite-precision and zero-spacing converter surprises without changing its rules.
pub(crate) fn warnings(screens: &[ScreenCfg]) -> Vec<String> {
    let mut warnings = Vec::new();
    if screens.iter().enumerate().any(|(i, a)| {
        screens[i + 1..]
            .iter()
            .any(|b| match (a.rect.and_then(expand), b.rect.and_then(expand)) {
                (Some(a), Some(b)) => overlaps(a, b),
                _ => false,
            })
    }) {
        warnings.push("Expanded layout rects overlap: the exact Synergy converter can derive extra directions at zero spacing; inspect derived_links or override the affected edges".into());
        warnings.push(
            "Layout overlaps that round to empty integer percentage ranges are omitted (frozen u8 range precision)"
                .into(),
        );
    }
    warnings
}

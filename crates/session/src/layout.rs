//! Screen-local coordinates and config-owned percentage links.
use openergy_config::{derive_links, Config, Corner, Edge};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}
impl Default for Rect {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            w: 1920,
            h: 1080,
        }
    }
}
impl Rect {
    pub fn clamp(self, x: i32, y: i32) -> (i32, i32) {
        (
            x.clamp(self.x, self.x.saturating_add(self.w.max(1) - 1)),
            y.clamp(self.y, self.y.saturating_add(self.h.max(1) - 1)),
        )
    }
    pub fn center(self) -> (i32, i32) {
        (self.x.saturating_add(self.w / 2), self.y.saturating_add(self.h / 2))
    }
}
/// Nearest point on any display, or on `fallback` when no display is known.
pub fn nearest_on_display(displays: &[Rect], fallback: Rect, x: i32, y: i32) -> (i32, i32) {
    displays
        .iter()
        .filter(|d| d.w > 0 && d.h > 0)
        .map(|d| d.clamp(x, y))
        .min_by_key(|(px, py)| {
            let (dx, dy) = (i64::from(px - x), i64::from(py - y));
            dx * dx + dy * dy
        })
        .unwrap_or_else(|| fallback.clamp(x, y))
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    pub from: usize,
    pub edge: Edge,
    pub from_range: (u8, u8),
    pub to: usize,
    pub to_range: (u8, u8),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError {
    Validation(String),
    NotImplemented,
}

#[derive(Clone)]
pub struct Layout {
    links: Vec<Link>,
    rects: Vec<Rect>,
    // Per-screen display rectangles, when known. An edge then spans only the
    // displays that touch it, not empty space in the bounding box.
    displays: Vec<Vec<Rect>>,
    delay_ms: u64,
    double_tap_ms: u64,
    corners: Vec<Corner>,
    corner_size: i32,
    screen_corners: Vec<Option<Vec<Corner>>>,
    screen_sizes: Vec<Option<i32>>,
    candidate: Option<(usize, Edge, usize, u64)>,
    tap: Option<(usize, Edge, u64, bool)>,
}
impl Layout {
    pub fn from_config(cfg: &Config) -> Result<Self, LayoutError> {
        let mut links = Vec::new();
        for l in derive_links(&cfg.screens) {
            let from = cfg.screens.iter().position(|s| s.name == l.from);
            let to = cfg.screens.iter().position(|s| s.name == l.to);
            if let (Some(from), Some(to)) = (from, to) {
                if l.from_range.0 >= l.from_range.1 || l.to_range.0 >= l.to_range.1 {
                    return Err(LayoutError::Validation("empty link range".into()));
                }
                if links.iter().any(|p: &Link| {
                    p.from == from
                        && p.edge == l.edge
                        && p.from_range.0 < l.from_range.1
                        && l.from_range.0 < p.from_range.1
                }) {
                    return Err(LayoutError::Validation("overlapping source link ranges".into()));
                }
                links.push(Link {
                    from,
                    edge: l.edge,
                    from_range: l.from_range,
                    to,
                    to_range: l.to_range,
                });
            }
        }
        Ok(Self {
            links,
            rects: cfg
                .screens
                .iter()
                .map(|s| {
                    s.rect.map_or_else(Rect::default, |r| Rect {
                        x: 0,
                        y: 0,
                        w: (r.w as i32).clamp(1, i16::MAX as i32),
                        h: (r.h as i32).clamp(1, i16::MAX as i32),
                    })
                })
                .collect(),
            displays: vec![Vec::new(); cfg.screens.len()],
            delay_ms: cfg.switch.delay_ms,
            double_tap_ms: cfg.switch.double_tap_ms,
            corners: cfg.switch.dead_corners.clone(),
            corner_size: cfg.switch.corner_size_px.min(i32::MAX as u32) as i32,
            screen_corners: cfg
                .screens
                .iter()
                .map(|s| s.options.as_ref().and_then(|o| o.dead_corners.clone()))
                .collect(),
            screen_sizes: cfg
                .screens
                .iter()
                .map(|s| {
                    s.options
                        .as_ref()
                        .and_then(|o| o.corner_size_px)
                        .map(|n| n.min(i32::MAX as u32) as i32)
                })
                .collect(),
            candidate: None,
            tap: None,
        })
    }
    pub(crate) fn empty() -> Self {
        Self {
            links: Vec::new(),
            rects: Vec::new(),
            displays: Vec::new(),
            delay_ms: 0,
            double_tap_ms: 0,
            corners: Vec::new(),
            corner_size: 0,
            screen_corners: Vec::new(),
            screen_sizes: Vec::new(),
            candidate: None,
            tap: None,
        }
    }
    pub fn set_rect(&mut self, screen: usize, rect: Rect) {
        if let Some(r) = self.rects.get_mut(screen) {
            *r = rect;
        }
        self.reset();
    }
    /// Record the displays of a screen, in the same coordinates as its rect.
    pub fn set_displays(&mut self, screen: usize, displays: &[Rect]) {
        if let Some(d) = self.displays.get_mut(screen) {
            if d.as_slice() != displays {
                *d = displays.to_vec();
                self.reset();
            }
        }
    }
    /// Intervals along `edge` that a cursor can reach, sorted and merged.
    /// Without display data, the whole bounding-box edge is one interval.
    fn spans(&self, screen: usize, edge: Edge) -> Vec<(i32, i32)> {
        let r = self.rect(screen);
        let mut spans = self
            .displays
            .get(screen)
            .into_iter()
            .flatten()
            .filter(|d| d.w > 0 && d.h > 0)
            .filter_map(|d| {
                let touches = match edge {
                    Edge::Left => d.x == r.x,
                    Edge::Right => d.x.saturating_add(d.w) == r.x.saturating_add(r.w),
                    Edge::Top => d.y == r.y,
                    Edge::Bottom => d.y.saturating_add(d.h) == r.y.saturating_add(r.h),
                };
                let (start, len, lo, hi) = match edge {
                    Edge::Left | Edge::Right => (d.y, d.h, r.y, r.y.saturating_add(r.h)),
                    _ => (d.x, d.w, r.x, r.x.saturating_add(r.w)),
                };
                let span = (start.max(lo), start.saturating_add(len).min(hi));
                (touches && span.0 < span.1).then_some(span)
            })
            .collect::<Vec<_>>();
        spans.sort_unstable();
        let mut merged: Vec<(i32, i32)> = Vec::new();
        for (a, b) in spans {
            match merged.last_mut() {
                Some(last) if a <= last.1 => last.1 = last.1.max(b),
                _ => merged.push((a, b)),
            }
        }
        if merged.is_empty() {
            merged.push(match edge {
                Edge::Left | Edge::Right => (r.y, r.y.saturating_add(r.h.max(1))),
                _ => (r.x, r.x.saturating_add(r.w.max(1))),
            });
        }
        merged
    }
    /// Pixel interval of an edge for a link range in percent of the full edge.
    fn link_interval(&self, screen: usize, edge: Edge, range: (u8, u8)) -> (i32, i32) {
        let r = self.rect(screen);
        let (start, extent) = match edge {
            Edge::Left | Edge::Right => (r.y, r.h.max(1)),
            _ => (r.x, r.w.max(1)),
        };
        let at = |p: u8| start.saturating_add((i64::from(extent) * i64::from(p) / 100) as i32);
        let (lo, hi) = (at(range.0), at(range.1));
        (lo, hi.max(lo.saturating_add(1)))
    }
    /// Reachable parts of a link interval. Link ranges come from the config
    /// layout and stay in full-edge terms; only movement inside a link skips
    /// the parts of the edge that no display covers.
    fn reachable(&self, screen: usize, edge: Edge, (lo, hi): (i32, i32)) -> Vec<(i32, i32)> {
        let clipped = self
            .spans(screen, edge)
            .into_iter()
            .map(|(a, b)| (a.max(lo), b.min(hi)))
            .filter(|(a, b)| a < b)
            .collect::<Vec<_>>();
        if clipped.is_empty() {
            vec![(lo, hi)]
        } else {
            clipped
        }
    }
    /// Fraction in [0, 1) of a coordinate along the reachable part of an interval.
    fn edge_fraction(&self, screen: usize, edge: Edge, interval: (i32, i32), coordinate: i32) -> f64 {
        let spans = self.reachable(screen, edge, interval);
        let total: i64 = spans.iter().map(|(a, b)| i64::from(b - a)).sum();
        let mut offset = 0i64;
        for (a, b) in &spans {
            if coordinate < *a {
                break;
            }
            if coordinate < *b {
                offset += i64::from(coordinate - a);
                break;
            }
            offset += i64::from(b - a);
        }
        (offset.min(total - 1).max(0) as f64) / (total.max(1) as f64)
    }
    /// Coordinate on the reachable part of an interval for a fraction in [0, 1].
    fn edge_coordinate(&self, screen: usize, edge: Edge, interval: (i32, i32), fraction: f64) -> i32 {
        let spans = self.reachable(screen, edge, interval);
        let total: i64 = spans.iter().map(|(a, b)| i64::from(b - a)).sum();
        let mut remaining = ((fraction.clamp(0.0, 1.0) * total as f64) as i64).min(total - 1).max(0);
        for (a, b) in &spans {
            let len = i64::from(b - a);
            if remaining < len {
                return a + remaining as i32;
            }
            remaining -= len;
        }
        spans.last().map_or(0, |(_, b)| b - 1)
    }
    /// Nearest point on a known display, so a position in a gap between
    /// displays lands where the OS would show the cursor.
    pub fn snap_to_display(&self, screen: usize, x: i32, y: i32) -> (i32, i32) {
        let displays = self.displays.get(screen).map(Vec::as_slice).unwrap_or_default();
        nearest_on_display(displays, self.rect(screen), x, y)
    }
    pub fn rect(&self, screen: usize) -> Rect {
        self.rects.get(screen).copied().unwrap_or_default()
    }
    pub fn waiting(&self) -> bool {
        self.candidate.is_some()
    }
    pub fn next_deadline(&self, now: u64) -> u64 {
        self.candidate
            .map(|(_, _, _, since)| since.saturating_add(self.delay_ms).max(now + 4))
            .unwrap_or(now + 1000)
    }
    pub fn reset(&mut self) {
        self.candidate = None;
        self.tap = None;
    }
    pub fn set_gates(&mut self, delay_ms: u64, double_tap_ms: u64, corner_mask: i32, size: i32) {
        self.delay_ms = delay_ms;
        self.double_tap_ms = double_tap_ms;
        self.corner_size = size.max(0);
        self.corners = [
            (1, Corner::TopLeft),
            (2, Corner::TopRight),
            (4, Corner::BottomLeft),
            (8, Corner::BottomRight),
        ]
        .into_iter()
        .filter_map(|(mask, c)| (corner_mask & mask != 0).then_some(c))
        .collect();
        self.reset();
    }
    pub fn switch_target(
        &mut self,
        current: usize,
        x: i32,
        y: i32,
        dx: i32,
        dy: i32,
        now_ms: u64,
    ) -> Option<(usize, i32, i32)> {
        let r = self.rect(current);
        let right = r.x.saturating_add(r.w - 1);
        let bottom = r.y.saturating_add(r.h - 1);
        let edge = if x <= r.x && dx < 0 {
            Some(Edge::Left)
        } else if x >= right && dx > 0 {
            Some(Edge::Right)
        } else if y <= r.y && dy < 0 {
            Some(Edge::Top)
        } else if y >= bottom && dy > 0 {
            Some(Edge::Bottom)
        } else {
            None
        };
        let Some(edge) = edge else {
            self.candidate = None;
            if let Some(tap) = self.tap.as_mut() {
                let retreated = match tap.1 {
                    Edge::Left => x >= r.x + 3,
                    Edge::Right => x <= right - 3,
                    Edge::Top => y >= r.y + 3,
                    Edge::Bottom => y <= bottom - 3,
                };
                if tap.0 == current && retreated {
                    tap.3 = true;
                }
            }
            return None;
        };
        let n = self
            .screen_sizes
            .get(current)
            .copied()
            .flatten()
            .unwrap_or(self.corner_size);
        let corners = self
            .screen_corners
            .get(current)
            .and_then(Option::as_ref)
            .unwrap_or(&self.corners);
        if corners.iter().any(|c| match c {
            Corner::TopLeft => {
                i64::from(x) < i64::from(r.x) + i64::from(n) && i64::from(y) < i64::from(r.y) + i64::from(n)
            }
            Corner::TopRight => {
                i64::from(x) > i64::from(right) - i64::from(n) && i64::from(y) < i64::from(r.y) + i64::from(n)
            }
            Corner::BottomLeft => {
                i64::from(x) < i64::from(r.x) + i64::from(n) && i64::from(y) > i64::from(bottom) - i64::from(n)
            }
            Corner::BottomRight => {
                i64::from(x) > i64::from(right) - i64::from(n) && i64::from(y) > i64::from(bottom) - i64::from(n)
            }
        }) {
            self.candidate = None;
            return None;
        }
        let along_edge = match edge {
            Edge::Left | Edge::Right => y,
            _ => x,
        };
        // Link selection uses the full edge, like the derived link ranges.
        let (coordinate, extent) = match edge {
            Edge::Left | Edge::Right => (y - r.y, r.h),
            _ => (x - r.x, r.w),
        };
        let fraction = f64::from(coordinate.clamp(0, extent - 1)) / f64::from(extent);
        let Some(link) = self
            .links
            .iter()
            .find(|l| {
                l.from == current
                    && l.edge == edge
                    && fraction * 100.0 >= f64::from(l.from_range.0)
                    && fraction * 100.0 < f64::from(l.from_range.1)
            })
            .cloned()
        else {
            self.candidate = None;
            return None;
        };
        let since = match self.candidate {
            Some((s, e, t, since)) if s == current && e == edge && t == link.to => since,
            _ => {
                self.candidate = Some((current, edge, link.to, now_ms));
                now_ms
            }
        };
        if now_ms.saturating_sub(since) < self.delay_ms {
            return None;
        }
        if self.double_tap_ms > 0 {
            match self.tap {
                Some((s, e, t, retreated))
                    if s == current && e == edge && retreated && now_ms.saturating_sub(t) <= self.double_tap_ms => {}
                Some((s, e, t, false))
                    if s == current && e == edge && now_ms.saturating_sub(t) <= self.double_tap_ms =>
                {
                    self.candidate = None;
                    return None;
                }
                _ => {
                    self.tap = Some((current, edge, now_ms, false));
                    // The next hit requires physical retreat and return. A
                    // timer replay must not renew this hit while stationary.
                    self.candidate = None;
                    return None;
                }
            }
        }
        let source = self.link_interval(current, edge, link.from_range);
        let f = self.edge_fraction(current, edge, source, along_edge);
        let target = self.rect(link.to);
        // The target's entry edge faces the source edge.
        let entry = match edge {
            Edge::Left => Edge::Right,
            Edge::Right => Edge::Left,
            Edge::Top => Edge::Bottom,
            Edge::Bottom => Edge::Top,
        };
        let destination = self.link_interval(link.to, entry, link.to_range);
        let along = self.edge_coordinate(link.to, entry, destination, f);
        let (tx, ty) = match edge {
            Edge::Left => (target.x + target.w - 1, along),
            Edge::Right => (target.x, along),
            Edge::Top => (along, target.y + target.h - 1),
            Edge::Bottom => (along, target.y),
        };
        self.reset();
        let (tx, ty) = self.snap_to_display(link.to, tx, ty);
        Some((link.to, tx, ty))
    }
    pub fn links(&self) -> &[Link] {
        &self.links
    }
}

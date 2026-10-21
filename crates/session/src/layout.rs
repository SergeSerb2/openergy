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

pub struct Layout {
    links: Vec<Link>,
    rects: Vec<Rect>,
    delay_ms: u64,
    double_tap_ms: u64,
    corners: Vec<Corner>,
    corner_size: i32,
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
            delay_ms: cfg.switch.delay_ms,
            double_tap_ms: cfg.switch.double_tap_ms,
            corners: cfg.switch.dead_corners.clone(),
            corner_size: cfg.switch.corner_size_px.min(i32::MAX as u32) as i32,
            candidate: None,
            tap: None,
        })
    }
    pub(crate) fn empty() -> Self {
        Self {
            links: Vec::new(),
            rects: Vec::new(),
            delay_ms: 0,
            double_tap_ms: 0,
            corners: Vec::new(),
            corner_size: 0,
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
    pub fn rect(&self, screen: usize) -> Rect {
        self.rects.get(screen).copied().unwrap_or_default()
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
            if x >= r.x + 3 && x <= right - 3 && y >= r.y + 3 && y <= bottom - 3 {
                if let Some(tap) = self.tap.as_mut() {
                    tap.3 = true;
                }
            }
            return None;
        };
        let n = self.corner_size;
        if self.corners.iter().any(|c| match c {
            Corner::TopLeft => x < r.x + n && y < r.y + n,
            Corner::TopRight => x > right - n && y < r.y + n,
            Corner::BottomLeft => x < r.x + n && y > bottom - n,
            Corner::BottomRight => x > right - n && y > bottom - n,
        }) {
            self.candidate = None;
            return None;
        }
        let (coordinate, extent) = match edge {
            Edge::Left | Edge::Right => (y - r.y, r.h),
            _ => (x - r.x, r.w),
        };
        let fraction = f64::from(coordinate.clamp(0, extent - 1)) / f64::from(extent);
        let link = self
            .links
            .iter()
            .find(|l| {
                l.from == current
                    && l.edge == edge
                    && fraction * 100.0 >= f64::from(l.from_range.0)
                    && fraction * 100.0 < f64::from(l.from_range.1)
            })?
            .clone();
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
                    return None
                }
                _ => {
                    self.tap = Some((current, edge, now_ms, false));
                    return None;
                }
            }
        }
        let f = ((fraction * 100.0 - f64::from(link.from_range.0)) / f64::from(link.from_range.1 - link.from_range.0))
            .clamp(0.0, 1.0);
        let target_f = (f64::from(link.to_range.0) + f * f64::from(link.to_range.1 - link.to_range.0)) / 100.0;
        let target = self.rect(link.to);
        let (tx, ty) = match edge {
            Edge::Left => (
                target.x + target.w - 1,
                target.y + (target_f * f64::from(target.h)) as i32,
            ),
            Edge::Right => (target.x, target.y + (target_f * f64::from(target.h)) as i32),
            Edge::Top => (
                target.x + (target_f * f64::from(target.w)) as i32,
                target.y + target.h - 1,
            ),
            Edge::Bottom => (target.x + (target_f * f64::from(target.w)) as i32, target.y),
        };
        self.reset();
        let (tx, ty) = target.clamp(tx, ty);
        Some((link.to, tx, ty))
    }
    pub fn links(&self) -> &[Link] {
        &self.links
    }
}

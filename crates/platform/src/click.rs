//! Original, platform-neutral implementation of docs/focus-fix.md.
use crate::ClickNumberer;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClickAssignment {
    pub number: i64,
    pub state: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClickError {
    InvalidButton,
    InvalidPosition,
    AlreadyPressed,
    NumberExhausted,
}
impl fmt::Display for ClickError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ClickError {}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ClickHistory {
    time_ms: Option<u64>,
    x: f64,
    y: f64,
    state: i64,
    dragged: bool,
}

impl Default for ClickNumberer {
    fn default() -> Self {
        Self::with_double_click_interval(500)
    }
}

impl ClickNumberer {
    pub fn with_double_click_interval(double_click_ms: u64) -> Self {
        Self {
            last_number: 0,
            pressed_number: [None; 8],
            pressed_click_state: [None; 8],
            history: [ClickHistory::default(); 8],
            double_click_ms,
            click_distance: 4.0,
            exhaustion_logged: false,
        }
    }

    /// Use monotonic milliseconds and screen points. No allocation or OS calls.
    /// Failure leaves all press and click-history state unchanged.
    pub fn on_down_at(
        &mut self,
        button: u8,
        seed: i64,
        now_ms: u64,
        x: f64,
        y: f64,
    ) -> Result<ClickAssignment, ClickError> {
        let index = usize::from(button);
        if index == 0 || index >= self.pressed_number.len() {
            return Err(ClickError::InvalidButton);
        }
        if !x.is_finite() || !y.is_finite() {
            return Err(ClickError::InvalidPosition);
        }
        if self.pressed_number[index].is_some() {
            return Err(ClickError::AlreadyPressed);
        }
        if self.last_number.max(seed) == i64::MAX {
            self.log_exhaustion();
            return Err(ClickError::NumberExhausted);
        }
        let previous = self.history[index];
        let timely = previous
            .time_ms
            .and_then(|time| now_ms.checked_sub(time))
            .is_some_and(|elapsed| elapsed <= self.double_click_ms);
        let close = within_distance(previous.x, previous.y, x, y, self.click_distance);
        let state = if timely && close && !previous.dragged {
            previous.state.saturating_add(1)
        } else {
            1
        };
        let number = self.on_down(button, seed);
        self.pressed_click_state[index] = Some(state);
        self.history[index] = ClickHistory {
            time_ms: Some(now_ms),
            x,
            y,
            state,
            dragged: false,
        };
        Ok(ClickAssignment { number, state })
    }

    /// Dragging outside four points breaks the NEXT multi-click sequence.
    /// The current press retains its original number and click count.
    pub fn on_drag_at(&mut self, button: u8, x: f64, y: f64) -> Option<ClickAssignment> {
        let assignment = self.assignment(button)?;
        let history = &mut self.history[usize::from(button)];
        if !within_distance(history.x, history.y, x, y, self.click_distance) {
            history.dragged = true;
        }
        Some(assignment)
    }

    /// Read before calling the frozen `on_up_clear` method.
    pub fn assignment(&self, button: u8) -> Option<ClickAssignment> {
        let index = usize::from(button);
        Some(ClickAssignment {
            number: self.pressed_number.get(index).copied().flatten()?,
            state: self.pressed_click_state.get(index).copied().flatten()?,
        })
    }

    /// End click chains after a session leave, without resetting the counter.
    pub fn reset_click_history(&mut self) {
        self.history = [ClickHistory::default(); 8];
    }

    pub(crate) fn log_exhaustion(&mut self) {
        if !self.exhaustion_logged {
            log::error!("mouse event number exhausted; refusing further numbered downs");
            self.exhaustion_logged = true;
        }
    }
}

fn within_distance(ax: f64, ay: f64, bx: f64, by: f64, distance: f64) -> bool {
    let dx = ax - bx;
    let dy = ay - by;
    dx * dx + dy * dy <= distance * distance
}

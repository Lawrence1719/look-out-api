use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Up,
    Down,
    #[default]
    Unknown,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub status: Status,
    pub consecutive_fail: u32,
    pub consecutive_ok: u32,
    pub last_changed_at: Option<String>,
    pub last_checked_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    Down,
    Recovered,
}

impl Transition {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Down => "DOWN",
            Self::Recovered => "RECOVERED",
        }
    }
}

pub fn advance(state: &mut State, ok: bool, failure: u32, recovery: u32) -> Option<Transition> {
    if ok {
        state.consecutive_fail = 0;
        state.consecutive_ok = state.consecutive_ok.saturating_add(1).min(recovery);
        match state.status {
            Status::Unknown => {
                state.status = Status::Up;
                None
            }
            Status::Down if state.consecutive_ok >= recovery => {
                state.status = Status::Up;
                Some(Transition::Recovered)
            }
            _ => None,
        }
    } else {
        state.consecutive_ok = 0;
        state.consecutive_fail = state.consecutive_fail.saturating_add(1).min(failure);
        if state.status != Status::Down && state.consecutive_fail >= failure {
            state.status = Status::Down;
            Some(Transition::Down)
        } else {
            None
        }
    }
}

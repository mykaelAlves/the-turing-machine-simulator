use crate::types::Configuration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HaltingState {
    Accept,
    Reject(HaltingStateReason),
}

#[derive(Debug, Clone)]
pub enum NtmHaltingState {
    Accept(Configuration),
    Reject(HaltingStateReason),
    Running,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HaltingStateReason {
    NoTransition,
    HitWall,
    FiniteTapeLimit,
    Unexpected(InternalHaltingStateReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InternalHaltingStateReason {
    ExceededMaxSteps,
    ExceededMaxTapeSize,
    ExceededMaxTapes,
    InvalidTransition,
}

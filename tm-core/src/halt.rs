#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HaltingState {
    Accept,
    Reject(HaltingStateReason),
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
    InvalidTransition,
}

use crate::tape::single::SingleTape;

/// Even if the machine is infinite, we need to set a maximum tape size to
/// prevent the simulator from running indefinitely. This is a reasonable
/// limit for most Turing machine simulations, as it allows us to explore a
/// wide range of configurations without risking infinite loops or excessive
/// memory usage.
///
/// Can be overriden by the user.
pub const MAX_TAPE_SIZE: u16 = 5_000;

/// Similarly, we need to set a maximum number of steps to prevent the
/// simulator from running indefinitely. This is especially important for
/// Turing machines that may not halt, as it allows us to limit the
/// execution time and resources used by the simulator.
///
/// Can be overriden by the user.
pub const MAX_STEPS: u16 = 10_000;

pub const N_TAPES: u8 = 4;
pub const N_HEADS: u8 = 4;

/// Represents a symbol on the tape. Using `Option<char>` to allow for a blank
/// as None.
pub type Symbol = Option<char>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Reading<S> {
    pub state: State,
    pub symbol: S,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Action<S, D> {
    pub next_state: State,
    pub write_symbol: S,
    pub direction: D,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Stay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct State(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MoveType {
    #[default]
    Strict, // classical, no staying in place
    NonStrict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TapeType {
    #[default]
    Single,

    MultiTape,

    MultiHead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TapeBoundary {
    #[default]
    Left,

    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TapeTheoreticalSize {
    Finite(u16),

    SemiInfinite(TapeBoundary),

    Infinite,
}

impl Default for TapeTheoreticalSize {
    fn default() -> Self {
        Self::SemiInfinite(TapeBoundary::default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrueBounds {
    pub true_tape_size: u16,
    pub max_steps: u16,
    pub max_tapes: u8,
    pub max_heads: u8,
}

impl Default for TrueBounds {
    fn default() -> Self {
        Self {
            true_tape_size: MAX_TAPE_SIZE,
            max_steps: MAX_STEPS,
            max_tapes: N_TAPES,
            max_heads: N_HEADS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TapeDelta {
    pub previous_state: State,
    pub overwritten_symbol: Symbol,
    pub direction_moved: Direction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MultiTapeDelta<const TAPES: usize> {
    pub previous_state: State,
    pub overwritten_symbols: [Symbol; TAPES],
    pub directions_moved: [Direction; TAPES],
}

#[derive(Debug, Clone)]
pub struct Configuration {
    pub tape: SingleTape,
    pub current_state: State,
    pub history: Vec<TapeDelta>,
}

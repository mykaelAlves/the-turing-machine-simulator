use std::collections::{HashMap, VecDeque};

pub mod default_tm_configs {
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

    /// The maximum number of tapes a Turing machine can have. This is a
    /// reasonable limit for most Turing machine simulations, as it allows us
    /// to explore a wide range of configurations without risking excessive
    /// complexity or resource usage.
    ///
    /// Can be overriden by the user.
    pub const MAX_TAPES: u8 = 4;

    /// The maximum number of heads a Turing machine can have. This is a
    /// reasonable limit for most Turing machine simulations, as it allows us
    /// to explore a wide range of configurations without risking excessive
    /// complexity or resource usage.
    ///
    /// Can be overriden by the user.
    pub const MAX_HEADS: u8 = 4;
}

pub type Symbol = Option<char>;

pub type SimpleReadingState = (State, Symbol);
pub type SimpleTransition = (State, Symbol, Direction);

pub type MultiTapeReadingState<const TAPES: u8> = (State, [Symbol; TAPES]);
pub type MultiTapeTransition<const TAPES: u8> = (State, [Symbol; TAPES], [Direction; TAPES]);

pub type MultiHeadReadingState<const HEADS: u8> = (State, [Symbol; HEADS]);
pub type MultiHeadTransition<const HEADS: u8> = (State, [Symbol; HEADS], [Direction; HEADS]);

pub type SingleTapeReadingState = (State, Symbol);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Stay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HaltingState {
    Accept,
    Reject(HaltingStateReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HaltingStateReason {
    NoTransition,
    HitWall,
    Unexpected(InternalHaltingStateReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InternalHaltingStateReason {
    ExceededMaxSteps,
    ExceededMaxTapeSize,
    InvalidTransition,
}

pub trait TuringMachine {
    fn step(&mut self);
    fn reset(&mut self);
    fn back(&mut self);
}

pub trait Tape {
    fn read(&self) -> Symbol;
    fn write(&mut self, symbol: Symbol);
    fn to_vec(&self) -> Vec<Symbol>;
}

pub struct State(pub u16);

pub struct SingleTape {
    /// Left of the head. The last element is the one immediately to the left of
    /// the head.
    pub left: Vec<Symbol>,

    /// The symbol under the head.
    pub head: Symbol,

    /// Right of the head. The last element is the one immediately to the right
    /// of the head.
    pub right: Vec<Symbol>,
}

pub struct MultiHeadTape<const HEADS: usize> {
    /// All symbols on the tape. The head positions are tracked separately, so
    /// this is just a flat vector of symbols. The head positions are used to
    /// determine which symbol each head is currently reading or writing.
    pub memory: Vec<Symbol>,

    /// Pointers to the current position of each head.
    pub head_positions: [usize; HEADS],

    /// Offset from the initial alphabet. Useful for seminfinite tapes, where we
    /// need to track how far we've moved from the initial position to know when
    /// hitting the wall.
    pub offset: isize,
}

use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
};

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

pub trait Computable {
    fn step(&mut self) -> Option<HaltingState>;
    fn reset(&mut self);
    fn back(&mut self);
}

pub trait Tape {
    type SymbolType;
    type ExportType;

    fn read(&self) -> Self::SymbolType;
    fn to_vec(&self) -> Vec<Self::ExportType>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct State(pub u16);

#[derive(Debug, Clone)]
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

impl Tape for SingleTape {
    type SymbolType = Symbol;
    type ExportType = Symbol;

    fn read(&self) -> Self::SymbolType {
        self.head
    }

    fn to_vec(&self) -> Vec<Self::ExportType> {
        let mut tape = self.left.clone();
        tape.push(self.head);
        tape.extend(self.right.iter().rev().cloned());

        tape
    }
}

pub struct MultiTape<const TAPES: usize>(pub [SingleTape; TAPES]);

impl<const TAPES: usize> Tape for MultiTape<TAPES> {
    type SymbolType = [Symbol; TAPES];
    type ExportType = [Symbol; TAPES];

    fn read(&self) -> Self::SymbolType {
        let mut symbols = [None; TAPES];
        for (i, tape) in self.0.iter().enumerate() {
            symbols[i] = tape.read();
        }
        symbols
    }

    fn to_vec(&self) -> Vec<Self::ExportType> {
        let tape_vectors: Vec<Vec<Symbol>> = self.0.iter().map(|tape| tape.to_vec()).collect();
        let max_len = tape_vectors
            .iter()
            .map(|tape| tape.len())
            .max()
            .unwrap_or(0);

        let mut tapes = Vec::with_capacity(max_len);
        for i in 0..max_len {
            let mut symbols = [None; TAPES];
            for (tape_idx, tape) in tape_vectors.iter().enumerate() {
                if let Some(symbol) = tape.get(i) {
                    symbols[tape_idx] = *symbol;
                }
            }
            tapes.push(symbols);
        }

        tapes
    }
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

impl<const HEADS: usize> Tape for MultiHeadTape<HEADS> {
    type SymbolType = [Symbol; HEADS];
    type ExportType = Symbol;

    fn read(&self) -> Self::SymbolType {
        let mut symbols = [None; HEADS];
        for (i, &pos) in self.head_positions.iter().enumerate() {
            if pos < self.memory.len() {
                symbols[i] = self.memory[pos];
            }
        }
        symbols
    }

    fn to_vec(&self) -> Vec<Self::ExportType> {
        self.memory.clone()
    }
}

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
pub enum TapeTheoraticalSize {
    Finite(u16),

    SemiInfinite(TapeBoundary),

    Infinite,
}

impl Default for TapeTheoraticalSize {
    fn default() -> Self {
        Self::SemiInfinite(TapeBoundary::default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TrueBounds {
    true_tape_size: u16,
    max_steps: u16,
    max_tapes: u8,
    max_heads: u8,
}

impl Default for TrueBounds {
    fn default() -> Self {
        Self {
            true_tape_size: MAX_TAPE_SIZE,
            max_steps: MAX_STEPS,
            max_tapes: MAX_TAPES,
            max_heads: MAX_HEADS,
        }
    }
}

/// Allows building incrementally, which is useful for the UI, where the user
/// will start with an empty machine.
#[derive(Debug, Clone)]
pub struct SingleTapeDTMBuilder {
    transitions: HashMap<SimpleReadingState, SimpleTransition>,
    initial_state: Option<State>,
    initial_tape: Option<SingleTape>,
    accepting_states: Vec<State>,
    tape: Option<SingleTape>,
    current_state: Option<State>,
    move_type: Option<MoveType>,
    tape_size: Option<TapeTheoraticalSize>,
    history: Vec<(SingleTape, State)>,
    true_bounds: TrueBounds,
}

impl Default for SingleTapeDTMBuilder {
    fn default() -> Self {
        Self {
            transitions: HashMap::new(),
            initial_state: None,
            initial_tape: None,
            accepting_states: Vec::new(),
            tape: None,
            current_state: None,
            move_type: None,
            tape_size: None,
            history: Vec::new(),
            true_bounds: TrueBounds::default(),
        }
    }
}

impl SingleTapeDTMBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> Option<SingleTapeDTM> {
        Some(SingleTapeDTM {
            transitions: self.transitions,
            initial_state: self.initial_state?,
            initial_tape: self.initial_tape.clone()?,
            accepting_states: self.accepting_states,
            tape: self.initial_tape?,
            current_state: self.initial_state?,
            move_type: self.move_type?,
            tape_size: self.tape_size?,
            history: Vec::new(),
            true_bounds: self.true_bounds,
        })
    }

    pub fn insert_transition(
        &mut self,
        reading_state: SimpleReadingState,
        transition: SimpleTransition,
    ) -> &mut Self {
        if self.transitions.contains_key(&reading_state) {
            return self;
        }

        self.transitions.insert(reading_state, transition);

        self
    }

    pub fn insert_transitions(
        &mut self,
        transitions: Vec<(SimpleReadingState, SimpleTransition)>,
    ) -> &mut Self {
        for (reading_state, transition) in transitions {
            if self.transitions.contains_key(&reading_state) {
                continue;
            }

            self.transitions.insert(reading_state, transition);
        }

        self
    }

    pub fn with_initial_state(&mut self, initial_state: State) -> &mut Self {
        self.initial_state = Some(initial_state);
        self.current_state = Some(initial_state);

        self
    }

    pub fn with_tape(&mut self, initial_tape: SingleTape) -> &mut Self {
        self.initial_tape = Some(initial_tape.clone());
        self.tape = Some(initial_tape);

        self
    }

    pub fn with_accepting_states(&mut self, accepting_states: Vec<State>) -> &mut Self {
        self.accepting_states = accepting_states;

        self
    }

    pub fn with_move_type(&mut self, move_type: MoveType) -> &mut Self {
        self.move_type = Some(move_type);

        self
    }

    pub fn with_tape_size(&mut self, tape_size: TapeTheoraticalSize) -> &mut Self {
        self.tape_size = Some(tape_size);

        self
    }
}

pub struct SingleTapeDTM {
    transitions: HashMap<SimpleReadingState, SimpleTransition>,
    initial_state: State,
    initial_tape: SingleTape,
    accepting_states: Vec<State>,
    tape: SingleTape,
    current_state: State,
    move_type: MoveType,
    tape_size: TapeTheoraticalSize,
    history: Vec<(SingleTape, State)>,
    true_bounds: TrueBounds,
}

impl Computable for SingleTapeDTM {
    fn step(&mut self) -> Option<HaltingState> {
        if self.accepting_states.contains(&self.current_state) {
            return Some(HaltingState::Accept);
        }

        if self.history.len() as u16 >= self.true_bounds.max_steps {
            return Some(HaltingState::Reject(HaltingStateReason::Unexpected(
                InternalHaltingStateReason::ExceededMaxSteps,
            )));
        }

        let current_size = self.tape.left.len() as u16 + self.tape.right.len() as u16 + 1;

        if current_size > self.true_bounds.true_tape_size {
            return Some(HaltingState::Reject(HaltingStateReason::Unexpected(
                InternalHaltingStateReason::ExceededMaxTapeSize,
            )));
        }

        if let TapeTheoraticalSize::Finite(max_limit) = self.tape_size {
            if current_size >= max_limit {
                return Some(HaltingState::Reject(HaltingStateReason::Unexpected(
                    InternalHaltingStateReason::ExceededMaxTapeSize,
                )));
            }
        }

        let current_symbol = self.tape.read();
        let reading_state = (self.current_state, current_symbol);

        if let Some(transition) = self.transitions.get(&reading_state) {
            let (new_state, new_symbol, direction) = *transition;

            self.history.push((self.tape.clone(), self.current_state));

            match direction {
                Direction::Left => {
                    if self.tape.left.is_empty() {
                        if self.tape_size == TapeTheoraticalSize::SemiInfinite(TapeBoundary::Left) {
                            self.history.pop();

                            return Some(HaltingState::Reject(HaltingStateReason::HitWall));
                        }
                    }

                    self.tape.right.push(new_symbol);
                    self.tape.head = self.tape.left.pop().unwrap_or(None);
                    self.current_state = new_state;

                    return None;
                }
                Direction::Right => {
                    if self.tape.right.is_empty() {
                        if self.tape_size == TapeTheoraticalSize::SemiInfinite(TapeBoundary::Right)
                        {
                            self.history.pop();

                            return Some(HaltingState::Reject(HaltingStateReason::HitWall));
                        }
                    }

                    self.tape.left.push(new_symbol);
                    self.tape.head = self.tape.right.pop().unwrap_or(None);
                    self.current_state = new_state;

                    return None;
                }
                Direction::Stay => {
                    if self.move_type == MoveType::Strict {
                        self.history.pop();

                        return Some(HaltingState::Reject(HaltingStateReason::Unexpected(
                            InternalHaltingStateReason::InvalidTransition,
                        )));
                    }

                    self.tape.head = new_symbol;
                    self.current_state = new_state;

                    return None;
                }
            }
        }

        Some(HaltingState::Reject(HaltingStateReason::NoTransition))
    }

    fn reset(&mut self) {
        self.tape = self.initial_tape.clone();
        self.current_state = self.initial_state;
        self.history.clear();
    }

    fn back(&mut self) {
        if let Some((tape, state)) = self.history.pop() {
            self.tape = tape;
            self.current_state = state;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_until_halt(tm: &mut SingleTapeDTM) -> HaltingState {
        loop {
            if let Some(halt_state) = tm.step() {
                return halt_state;
            }
        }
    }

    #[test]
    fn test_bit_inverter_acceptance() {
        let state_0 = State(0);
        let accept_state = State(99);

        let initial_tape = SingleTape {
            left: vec![],
            head: Some('1'),
            right: vec![Some('1'), Some('0')], 
        };

        let mut builder = SingleTapeDTMBuilder::new();
        builder
            .with_initial_state(state_0)
            .with_accepting_states(vec![accept_state])
            .with_tape(initial_tape)
            .with_move_type(MoveType::Strict)
            .with_tape_size(TapeTheoraticalSize::SemiInfinite(TapeBoundary::Left))
            .insert_transition((state_0, Some('1')), (state_0, Some('0'), Direction::Right))
            .insert_transition((state_0, Some('0')), (state_0, Some('1'), Direction::Right))
            .insert_transition((state_0, None), (accept_state, None, Direction::Right));

        let mut tm = builder.build().expect("Falha ao construir a MT");
        let result = run_until_halt(&mut tm);

        assert_eq!(result, HaltingState::Accept);
        assert_eq!(tm.current_state, accept_state);

        let final_tape_visual = tm.tape.to_vec();
        assert_eq!(
            final_tape_visual, 
            vec![Some('0'), Some('1'), Some('0'), None, None]
        );
    }
}

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

pub trait Computable {
    fn run(&mut self) -> HaltingState;
    fn run_once(&mut self) -> Option<HaltingState>;
    fn reset(&mut self);
    fn back(&mut self);
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

impl SingleTape {
    pub fn read(&self) -> Symbol {
        self.head
    }

    pub fn to_vec(&self) -> Vec<Symbol> {
        let mut tape = self.left.clone();
        tape.push(self.head);
        tape.extend(self.right.iter().rev().cloned());

        tape
    }
}

#[derive(Debug, Clone)]
pub struct MultiTape<const TAPES: usize>(pub [SingleTape; TAPES]);

impl<const TAPES: usize> MultiTape<TAPES> {
    pub fn read(&self) -> [Symbol; TAPES] {
        let mut symbols = [None; TAPES];
        for (i, tape) in self.0.iter().enumerate() {
            symbols[i] = tape.read();
        }
        symbols
    }

    pub fn to_vecs(&self) -> Vec<Vec<Symbol>> {
        self.0.iter().map(|tape| tape.to_vec()).collect()
    }
}

#[derive(Debug, Clone)]
pub struct MultiHeadTape<const HEADS: usize> {
    /// All symbols on the tape. The head positions are tracked separately, so
    /// this is just a flat vector of symbols. The head positions are used to
    /// determine which symbol each head is currently reading or writing.
    pub memory: VecDeque<Symbol>,

    /// Pointers to the current position of each head.
    pub head_positions: [usize; HEADS],

    /// Offset from the initial alphabet. Useful for seminfinite tapes, where we
    /// need to track how far we've moved from the initial position to know when
    /// hitting the wall.
    pub offset: isize,
}

impl<const HEADS: usize> MultiHeadTape<HEADS> {
    pub fn read(&self) -> [Symbol; HEADS] {
        let mut symbols = [None; HEADS];
        for (i, &pos) in self.head_positions.iter().enumerate() {
            if pos < self.memory.len() {
                symbols[i] = self.memory[pos];
            }
        }
        symbols
    }

    pub fn to_vec(&self) -> VecDeque<Symbol> {
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

/// Allows building incrementally, which is useful for the UI, where the user
/// will start with an empty machine.
#[derive(Debug, Clone)]
pub struct SingleTapeDTMBuilder {
    transitions: HashMap<Reading<Symbol>, Action<Symbol, Direction>>,
    initial_state: Option<State>,
    initial_tape: Option<SingleTape>,
    accepting_states: Vec<State>,
    tape: Option<SingleTape>,
    current_state: Option<State>,
    move_type: Option<MoveType>,
    tape_size: Option<TapeTheoraticalSize>,
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
        reading_state: Reading<Symbol>,
        transition: Action<Symbol, Direction>,
    ) -> &mut Self {
        if self.transitions.contains_key(&reading_state) {
            return self;
        }

        self.transitions.insert(reading_state, transition);

        self
    }

    pub fn insert_transitions(
        &mut self,
        transitions: Vec<(Reading<Symbol>, Action<Symbol, Direction>)>,
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

    pub fn with_bounds(&mut self, bounds: TrueBounds) -> &mut Self {
        self.true_bounds = bounds;

        self
    }
}

pub struct SingleTapeDTM {
    transitions: HashMap<Reading<Symbol>, Action<Symbol, Direction>>,
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
    fn run_once(&mut self) -> Option<HaltingState> {
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
                return Some(HaltingState::Reject(HaltingStateReason::FiniteTapeLimit));
            }
        }

        let current_symbol = self.tape.read();
        let reading_state = Reading {
            state: self.current_state,
            symbol: current_symbol,
        };

        if let Some(transition) = self.transitions.get(&reading_state) {
            let (new_state, new_symbol, direction) = (
                transition.next_state,
                transition.write_symbol,
                transition.direction,
            );

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

    fn run(&mut self) -> HaltingState {
        loop {
            if let Some(halt_state) = self.run_once() {
                return halt_state;
            }
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.tape = self.initial_tape.clone();
        self.current_state = self.initial_state;
        self.history.clear();
    }

    #[inline]
    fn back(&mut self) {
        if let Some((tape, state)) = self.history.pop() {
            self.tape = tape;
            self.current_state = state;
        }
    }
}

pub struct MultiTapeDTMBuilder<const TAPES: usize> {
    transitions: HashMap<Reading<[Symbol; TAPES]>, Action<[Symbol; TAPES], [Direction; TAPES]>>,
    initial_state: Option<State>,
    initial_tape: Option<MultiTape<TAPES>>,
    accepting_states: Vec<State>,
    tape: Option<MultiTape<TAPES>>,
    current_state: Option<State>,
    move_type: Option<MoveType>,
    tape_size: Option<TapeTheoraticalSize>,
    true_bounds: TrueBounds,
}

impl<const TAPES: usize> MultiTapeDTMBuilder<TAPES> {
    pub fn new() -> Self {
        Self {
            transitions: HashMap::new(),
            initial_state: None,
            initial_tape: None,
            accepting_states: Vec::new(),
            tape: None,
            current_state: None,
            move_type: None,
            tape_size: None,
            true_bounds: TrueBounds::default(),
        }
    }

    pub fn build(self) -> Option<MultiTapeDTM<TAPES>> {
        Some(MultiTapeDTM {
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
        reading_state: Reading<[Symbol; TAPES]>,
        transition: Action<[Symbol; TAPES], [Direction; TAPES]>,
    ) -> &mut Self {
        if !self.transitions.contains_key(&reading_state) {
            self.transitions.insert(reading_state, transition);
        }
        self
    }

    pub fn insert_transitions(
        &mut self,
        transitions: Vec<(
            Reading<[Symbol; TAPES]>,
            Action<[Symbol; TAPES], [Direction; TAPES]>,
        )>,
    ) -> &mut Self {
        for (reading_state, transition) in transitions {
            self.insert_transition(reading_state, transition);
        }
        self
    }

    pub fn with_initial_state(&mut self, initial_state: State) -> &mut Self {
        self.initial_state = Some(initial_state);
        self.current_state = Some(initial_state);

        self
    }

    pub fn with_tapes(&mut self, initial_tape: MultiTape<TAPES>) -> &mut Self {
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

    pub fn with_bounds(&mut self, bounds: TrueBounds) -> &mut Self {
        self.true_bounds = bounds;

        self
    }
}

pub struct MultiTapeDTM<const TAPES: usize> {
    transitions: HashMap<Reading<[Symbol; TAPES]>, Action<[Symbol; TAPES], [Direction; TAPES]>>,
    initial_state: State,
    initial_tape: MultiTape<TAPES>,
    accepting_states: Vec<State>,
    tape: MultiTape<TAPES>,
    current_state: State,
    move_type: MoveType,
    tape_size: TapeTheoraticalSize,
    history: Vec<(MultiTape<TAPES>, State)>,
    true_bounds: TrueBounds,
}

impl<const TAPES: usize> Computable for MultiTapeDTM<TAPES> {
    fn run_once(&mut self) -> Option<HaltingState> {
        if self.accepting_states.contains(&self.current_state) {
            return Some(HaltingState::Accept);
        }

        if self.history.len() as u16 >= self.true_bounds.max_steps {
            return Some(HaltingState::Reject(HaltingStateReason::Unexpected(
                InternalHaltingStateReason::ExceededMaxSteps,
            )));
        }

        let mut max_current_size = 0;
        for t in &self.tape.0 {
            let size = t.left.len() as u16 + t.right.len() as u16 + 1;
            if size > max_current_size {
                max_current_size = size;
            }
        }

        if max_current_size > self.true_bounds.true_tape_size {
            return Some(HaltingState::Reject(HaltingStateReason::Unexpected(
                InternalHaltingStateReason::ExceededMaxTapeSize,
            )));
        }

        if let TapeTheoraticalSize::Finite(max_limit) = self.tape_size {
            if max_current_size >= max_limit {
                return Some(HaltingState::Reject(HaltingStateReason::FiniteTapeLimit));
            }
        }

        let current_symbols = self.tape.read();
        let reading_state = Reading {
            state: self.current_state,
            symbol: current_symbols,
        };

        if let Some(transition) = self.transitions.get(&reading_state) {
            let next_state = transition.next_state;
            let write_symbols = transition.write_symbol;
            let directions = transition.direction;

            self.history.push((self.tape.clone(), self.current_state));

            for i in 0..TAPES {
                let single_tape = &mut self.tape.0[i];
                let new_symbol = write_symbols[i];

                match directions[i] {
                    Direction::Left => {
                        if single_tape.left.is_empty() {
                            if self.tape_size
                                == TapeTheoraticalSize::SemiInfinite(TapeBoundary::Left)
                            {
                                let (old_tape, _) = self.history.pop().unwrap();
                                self.tape = old_tape;

                                return Some(HaltingState::Reject(HaltingStateReason::HitWall));
                            }
                        }

                        single_tape.right.push(new_symbol);
                        single_tape.head = single_tape.left.pop().unwrap_or(None);
                    }
                    Direction::Right => {
                        if single_tape.right.is_empty() {
                            if self.tape_size
                                == TapeTheoraticalSize::SemiInfinite(TapeBoundary::Right)
                            {
                                let (old_tape, _) = self.history.pop().unwrap();
                                self.tape = old_tape;

                                return Some(HaltingState::Reject(HaltingStateReason::HitWall));
                            }
                        }

                        single_tape.left.push(new_symbol);
                        single_tape.head = single_tape.right.pop().unwrap_or(None);
                    }
                    Direction::Stay => {
                        if self.move_type == MoveType::Strict {
                            let (old_tape, _) = self.history.pop().unwrap();
                            self.tape = old_tape;

                            return Some(HaltingState::Reject(HaltingStateReason::Unexpected(
                                InternalHaltingStateReason::InvalidTransition,
                            )));
                        }

                        single_tape.head = new_symbol;
                    }
                }
            }

            self.current_state = next_state;

            return None;
        }

        Some(HaltingState::Reject(HaltingStateReason::NoTransition))
    }

    fn run(&mut self) -> HaltingState {
        loop {
            if let Some(halt_state) = self.run_once() {
                return halt_state;
            }
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.tape = self.initial_tape.clone();
        self.current_state = self.initial_state;
        self.history.clear();
    }

    #[inline]
    fn back(&mut self) {
        if let Some((tape, state)) = self.history.pop() {
            self.tape = tape;
            self.current_state = state;
        }
    }
}

pub struct MultiHeadDTMBuilder<const HEADS: usize> {
    transitions: HashMap<Reading<[Symbol; HEADS]>, Action<[Symbol; HEADS], [Direction; HEADS]>>,
    initial_state: Option<State>,
    initial_tape: Option<MultiHeadTape<HEADS>>,
    accepting_states: Vec<State>,
    tape: Option<MultiHeadTape<HEADS>>,
    current_state: Option<State>,
    move_type: Option<MoveType>,
    tape_size: Option<TapeTheoraticalSize>,
    true_bounds: TrueBounds,
}

impl<const HEADS: usize> MultiHeadDTMBuilder<HEADS> {
    pub fn new() -> Self {
        Self {
            transitions: HashMap::new(),
            initial_state: None,
            initial_tape: None,
            accepting_states: Vec::new(),
            tape: None,
            current_state: None,
            move_type: None,
            tape_size: None,
            true_bounds: TrueBounds::default(),
        }
    }

    pub fn build(self) -> Option<MultiHeadDTM<HEADS>> {
        Some(MultiHeadDTM {
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
        reading_state: Reading<[Symbol; HEADS]>,
        transition: Action<[Symbol; HEADS], [Direction; HEADS]>,
    ) -> &mut Self {
        if !self.transitions.contains_key(&reading_state) {
            self.transitions.insert(reading_state, transition);
        }

        self
    }

    pub fn insert_transitions(
        &mut self,
        transitions: Vec<(
            Reading<[Symbol; HEADS]>,
            Action<[Symbol; HEADS], [Direction; HEADS]>,
        )>,
    ) -> &mut Self {
        for (reading_state, transition) in transitions {
            self.insert_transition(reading_state, transition);
        }

        self
    }

    pub fn with_initial_state(&mut self, initial_state: State) -> &mut Self {
        self.initial_state = Some(initial_state);
        self.current_state = Some(initial_state);

        self
    }

    pub fn with_tape(&mut self, initial_tape: MultiHeadTape<HEADS>) -> &mut Self {
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

    pub fn with_bounds(&mut self, bounds: TrueBounds) -> &mut Self {
        self.true_bounds = bounds;

        self
    }
}

pub struct MultiHeadDTM<const HEADS: usize> {
    transitions: HashMap<Reading<[Symbol; HEADS]>, Action<[Symbol; HEADS], [Direction; HEADS]>>,
    initial_state: State,
    initial_tape: MultiHeadTape<HEADS>,
    accepting_states: Vec<State>,
    tape: MultiHeadTape<HEADS>,
    current_state: State,
    move_type: MoveType,
    tape_size: TapeTheoraticalSize,
    history: Vec<(MultiHeadTape<HEADS>, State)>,
    true_bounds: TrueBounds,
}

impl<const HEADS: usize> Computable for MultiHeadDTM<HEADS> {
    fn run_once(&mut self) -> Option<HaltingState> {
        if self.accepting_states.contains(&self.current_state) {
            return Some(HaltingState::Accept);
        }

        if self.history.len() as u16 >= self.true_bounds.max_steps {
            return Some(HaltingState::Reject(HaltingStateReason::Unexpected(
                InternalHaltingStateReason::ExceededMaxSteps,
            )));
        }

        let current_size = self.tape.memory.len() as u16;

        if current_size > self.true_bounds.true_tape_size {
            return Some(HaltingState::Reject(HaltingStateReason::Unexpected(
                InternalHaltingStateReason::ExceededMaxTapeSize,
            )));
        }

        if let TapeTheoraticalSize::Finite(max_limit) = self.tape_size {
            if current_size >= max_limit {
                return Some(HaltingState::Reject(HaltingStateReason::FiniteTapeLimit));
            }
        }

        let current_symbols = self.tape.read();
        let reading_state = Reading {
            state: self.current_state,
            symbol: current_symbols,
        };

        if let Some(transition) = self.transitions.get(&reading_state) {
            self.history.push((self.tape.clone(), self.current_state));

            for i in 0..HEADS {
                let pos = self.tape.head_positions[i];

                self.tape.memory[pos] = transition.write_symbol[i];
            }

            for i in 0..HEADS {
                match transition.direction[i] {
                    Direction::Left => {
                        if self.tape.head_positions[i] == 0 {
                            if self.tape_size
                                == TapeTheoraticalSize::SemiInfinite(TapeBoundary::Left)
                            {
                                let (old_tape, _) = self.history.pop().unwrap();
                                self.tape = old_tape;
                                return Some(HaltingState::Reject(HaltingStateReason::HitWall));
                            }

                            self.tape.memory.push_front(None);
                            self.tape.offset -= 1;

                            for pos in self.tape.head_positions.iter_mut() {
                                *pos += 1;
                            }

                            self.tape.head_positions[i] -= 1;
                        } else {
                            self.tape.head_positions[i] -= 1;
                        }
                    }
                    Direction::Right => {
                        if self.tape.head_positions[i] == self.tape.memory.len() - 1 {
                            if self.tape_size
                                == TapeTheoraticalSize::SemiInfinite(TapeBoundary::Right)
                            {
                                let (old_tape, _) = self.history.pop().unwrap();
                                self.tape = old_tape;

                                return Some(HaltingState::Reject(HaltingStateReason::HitWall));
                            }

                            self.tape.memory.push_back(None);
                        }
                        self.tape.head_positions[i] += 1;
                    }
                    Direction::Stay => {
                        if self.move_type == MoveType::Strict {
                            let (old_tape, _) = self.history.pop().unwrap();
                            self.tape = old_tape;

                            return Some(HaltingState::Reject(HaltingStateReason::Unexpected(
                                InternalHaltingStateReason::InvalidTransition,
                            )));
                        }
                    }
                }
            }

            self.current_state = transition.next_state;

            return None;
        }

        Some(HaltingState::Reject(HaltingStateReason::NoTransition))
    }

    fn run(&mut self) -> HaltingState {
        loop {
            if let Some(halt_state) = self.run_once() {
                return halt_state;
            }
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.tape = self.initial_tape.clone();
        self.current_state = self.initial_state;
        self.history.clear();
    }

    #[inline]
    fn back(&mut self) {
        if let Some((tape, state)) = self.history.pop() {
            self.tape = tape;
            self.current_state = state;
        }
    }
}

pub struct SingleTapeNTM {
    // TODO
}

pub struct SingleTapeNTMBuilder {
    // TODO
}

pub struct MultiTapeNTM<const TAPES: usize> {
    // TODO
}

pub struct MultiTapeNTMBuilder<const TAPES: usize> {
    // TODO
}

pub struct MultiHeadNTM<const HEADS: usize> {
    // TODO
}

pub struct MultiHeadNTMBuilder<const HEADS: usize> {
    // TODO
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_until_halt(tm: &mut SingleTapeDTM) -> HaltingState {
        loop {
            if let Some(halt_state) = tm.run_once() {
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
            .insert_transition(
                Reading {
                    state: state_0,
                    symbol: Some('1'),
                },
                Action {
                    next_state: state_0,
                    write_symbol: Some('0'),
                    direction: Direction::Right,
                },
            )
            .insert_transition(
                Reading {
                    state: state_0,
                    symbol: Some('0'),
                },
                Action {
                    next_state: state_0,
                    write_symbol: Some('1'),
                    direction: Direction::Right,
                },
            )
            .insert_transition(
                Reading {
                    state: state_0,
                    symbol: None,
                },
                Action {
                    next_state: accept_state,
                    write_symbol: None,
                    direction: Direction::Right,
                },
            );

        let mut tm = builder.build().expect("Failed to build Turing machine");
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

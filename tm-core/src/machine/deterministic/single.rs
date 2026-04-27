use rustc_hash::FxHashMap as HashMap;

use crate::{
    halt::{HaltingState, HaltingStateReason, InternalHaltingStateReason},
    machine::Computable,
    tape::single::SingleTape,
    types::{
        Action, Direction, MoveType, Reading, State, Symbol, TapeBoundary, TapeTheoreticalSize,
        TrueBounds,
    },
};

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
    tape_size: Option<TapeTheoreticalSize>,
    true_bounds: TrueBounds,
}

impl Default for SingleTapeDTMBuilder {
    fn default() -> Self {
        Self {
            transitions: HashMap::default(),
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

    pub fn with_tape_size(&mut self, tape_size: TapeTheoreticalSize) -> &mut Self {
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
    tape_size: TapeTheoreticalSize,
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

        if let TapeTheoreticalSize::Finite(max_limit) = self.tape_size {
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
                        if self.tape_size == TapeTheoreticalSize::SemiInfinite(TapeBoundary::Left) {
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
                        if self.tape_size == TapeTheoreticalSize::SemiInfinite(TapeBoundary::Right)
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

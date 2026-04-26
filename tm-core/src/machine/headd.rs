use rustc_hash::FxHashMap as HashMap;

use crate::{
    halt::{HaltingState, HaltingStateReason, InternalHaltingStateReason},
    machine::Computable,
    tape::head::MultiHeadTape,
    types::{
        Action, Direction, MoveType, Reading, State, Symbol, TapeBoundary, TapeTheoreticalSize,
        TrueBounds,
    },
};

pub struct MultiHeadDTMBuilder<const HEADS: usize> {
    transitions: HashMap<Reading<[Symbol; HEADS]>, Action<[Symbol; HEADS], [Direction; HEADS]>>,
    initial_state: Option<State>,
    initial_tape: Option<MultiHeadTape<HEADS>>,
    accepting_states: Vec<State>,
    tape: Option<MultiHeadTape<HEADS>>,
    current_state: Option<State>,
    move_type: Option<MoveType>,
    tape_size: Option<TapeTheoreticalSize>,
    true_bounds: TrueBounds,
}

impl<const HEADS: usize> MultiHeadDTMBuilder<HEADS> {
    pub fn new() -> Self {
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

    pub fn with_tape_size(&mut self, tape_size: TapeTheoreticalSize) -> &mut Self {
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
    tape_size: TapeTheoreticalSize,
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

        if let TapeTheoreticalSize::Finite(max_limit) = self.tape_size {
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
                                == TapeTheoreticalSize::SemiInfinite(TapeBoundary::Left)
                            {
                                let (old_tape, old_state) = self.history.pop().unwrap();
                                self.tape = old_tape;
                                self.current_state = old_state;
                                
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
                                == TapeTheoreticalSize::SemiInfinite(TapeBoundary::Right)
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

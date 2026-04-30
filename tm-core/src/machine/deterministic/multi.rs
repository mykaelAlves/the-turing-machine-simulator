use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
    halt::{HaltingState, HaltingStateReason, InternalHaltingStateReason},
    machine::Computable,
    tape::multi::MultiTape,
    types::{
        Action, Direction, MoveType, MultiTapeDelta, Reading, State, Symbol, TapeBoundary,
        TapeTheoreticalSize, TrueBounds,
    },
};

pub struct MultiTapeDTMBuilder<const TAPES: usize> {
    transitions: HashMap<Reading<[Symbol; TAPES]>, Action<[Symbol; TAPES], [Direction; TAPES]>>,
    initial_state: Option<State>,
    initial_tape: Option<MultiTape<TAPES>>,
    accepting_states: HashSet<State>,
    tape: Option<MultiTape<TAPES>>,
    current_state: Option<State>,
    move_type: Option<MoveType>,
    tape_size: Option<TapeTheoreticalSize>,
    true_bounds: TrueBounds,
}

impl<const TAPES: usize> MultiTapeDTMBuilder<TAPES> {
    pub fn new() -> Self {
        Self {
            transitions: HashMap::default(),
            initial_state: None,
            initial_tape: None,
            accepting_states: HashSet::default(),
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
        self.accepting_states = accepting_states.into_iter().collect();

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

pub struct MultiTapeDTM<const TAPES: usize> {
    transitions: HashMap<Reading<[Symbol; TAPES]>, Action<[Symbol; TAPES], [Direction; TAPES]>>,
    initial_state: State,
    initial_tape: MultiTape<TAPES>,
    accepting_states: HashSet<State>,
    tape: MultiTape<TAPES>,
    current_state: State,
    move_type: MoveType,
    tape_size: TapeTheoreticalSize,
    history: Vec<MultiTapeDelta<TAPES>>,
    true_bounds: TrueBounds,
}

impl<const TAPES: usize> Computable for MultiTapeDTM<TAPES> {
    fn run_once(&mut self) -> Option<HaltingState> {
        self.check_bounds()?;

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
                                == TapeTheoreticalSize::SemiInfinite(TapeBoundary::Left)
                            {
                                let (old_tape, old_state) = self.history.pop().unwrap();
                                self.tape = old_tape;
                                self.current_state = old_state;

                                return Some(HaltingState::Reject(HaltingStateReason::HitWall));
                            }
                        }

                        single_tape.right.push(new_symbol);
                        single_tape.head = single_tape.left.pop().unwrap_or(None);
                    }
                    Direction::Right => {
                        if single_tape.right.is_empty() {
                            if self.tape_size
                                == TapeTheoreticalSize::SemiInfinite(TapeBoundary::Right)
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

    fn check_bounds(&self) -> Option<HaltingState> {
        if self.true_bounds.max_tapes < TAPES as u8 {
            return Some(HaltingState::Reject(HaltingStateReason::Unexpected(
                InternalHaltingStateReason::ExceededMaxTapes,
            )));
        }

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

        if let TapeTheoreticalSize::Finite(max_limit) = self.tape_size {
            if max_current_size >= max_limit {
                return Some(HaltingState::Reject(HaltingStateReason::FiniteTapeLimit));
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

#[cfg(test)]
mod tests {
    use super::*;
    fn test() {
        let tapes = 3;
        let tm = MultiTapeDTMBuilder::<tapes>::new()
            .with_initial_state(0)
            .with_tapes(MultiTape::new(vec![vec![], vec![]], vec![None, None]))
            .with_accepting_states(vec![1])
            .with_move_type(MoveType::Strict)
            .with_tape_size(TapeTheoreticalSize::SemiInfinite(TapeBoundary::Right))
            .with_bounds(TrueBounds {
                max_tapes: 2,
                max_steps: 100,
                true_tape_size: 100,
            })
            .insert_transition(
                Reading {
                    state: 0,
                    symbol: [None, None],
                },
                Action {
                    next_state: 1,
                    write_symbol: [Some(1), Some(0)],
                    direction: [Direction::Right, Direction::Right],
                },
            )
            .build()
            .unwrap();
    }
}

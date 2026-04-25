use std::collections::HashMap;

pub const MAX_DEFAULT_TAPE_SIZE: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Left,
    Right,
    Stay
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HaltState {
    Accept,
    Reject(RejectReason)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TuringMachineType {
    Deterministic(Characteristics),
    NonDeterministic(Characteristics)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Characteristics {
    pub move_type: MoveType,
    pub tape_type: TapeType,
    pub machine_subtype: TuringMachineSubtype
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoveType {
    Strict, // cant stay still as in the classic definition of a Turing machine
    NonStrict
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TapeType {
    Infinite,
    SemiInfinite(TapeBoundary),
    Finite(usize)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TapeBoundary {
    Left,
    Right
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TuringMachineSubtype {
    SingleTape,
    MultiTape,
    MultiHead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RejectReason {
    NoTransition,
    InvalidTransition,
    TapeOverflow,
    Timeout,
    HitWall,
    Unexpected(InternalError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InternalError {

}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct State(pub u16);

pub type Symbol = Option<char>;

pub type ReadingState = (State, Symbol);
pub type Transition = (State, Symbol, Direction);

#[derive(Debug, Clone)]
pub struct Tape {
    pub left: Vec<Symbol>,
    pub head: Symbol,
    pub right: Vec<Symbol>,
    pub size: usize,
}

impl Tape {
    pub fn new() -> Self {
        Self {
            left: Vec::new(),
            head: None,
            right: Vec::new(),
            size: MAX_DEFAULT_TAPE_SIZE
        }
    }

    pub fn read(&self) -> Vec<Symbol> {
        let mut full_tape = self.left.clone();
        full_tape.push(self.head);
        full_tape.extend(self.right.iter().rev().cloned());
        
        full_tape
    }

    pub fn write(&mut self, symbol: Symbol) {
        self.head = symbol;
    }
}

pub struct TuringMachineBuilder {
    tape: Option<Tape>,
    initial_tape: Option<Tape>,
    initial_state: Option<State>,
    current_state: Option<State>,
    accept_states: Vec<State>,
    transitions: HashMap<ReadingState, Vec<Transition>>,
    machine_type: Option<TuringMachineType>
}

impl TuringMachineBuilder {
    pub fn new() -> Self {
        Self {
            tape: None,
            initial_tape: None,
            initial_state: None,
            current_state: None,
            machine_type: None,
            accept_states: Vec::new(),
            transitions: HashMap::new()
        }
    }

    pub fn with_tape(&mut self, tape: Tape) -> &mut Self {
        self.tape = Some(tape.clone());
        self.initial_tape = Some(tape);

        self
    }

    pub fn with_initial_state(&mut self, state: State) -> &mut Self {
        self.initial_state = Some(state);
        self.current_state = Some(state);

        self
    }

    pub fn with_accept_state(&mut self, state: State) -> &mut Self {
        self.accept_states.push(state);

        self
    }

    pub fn with_accept_states(&mut self, states: Vec<State>) -> &mut Self {
        self.accept_states.extend(states);

        self
    }

    pub fn with_machine_type(&mut self, machine_type: TuringMachineType) -> &mut Self {
        self.machine_type = Some(machine_type);

        self
    }

    pub fn insert_transition(&mut self, reading_state: ReadingState, transition: Transition) -> &mut Self {
        if self.transitions.contains_key(&reading_state) {
            self.transitions.get_mut(&reading_state).unwrap().push(transition);
        } else {
            self.transitions.insert(reading_state, vec![transition]);
        }

        self
    }

    pub fn insert_transitions(&mut self, transitions: Vec<(ReadingState, Transition)>) -> &mut Self {
        for (reading_state, transition) in transitions {
            self.insert_transition(reading_state, transition);
        }

        self
    }

    pub fn build(self) -> TuringMachine {
        TuringMachine {
            tape: self.tape.expect("Tape must be set"),
            initial_tape: self.initial_tape.expect("Initial tape must be set"),
            initial_state: self.initial_state.expect("Initial state must be set"),
            current_state: self.current_state.expect("Current state must be set"),
            accept_states: self.accept_states,
            transitions: self.transitions,
            history: Vec::new(),
            machine_type: self.machine_type.expect("Machine type must be set")
        }
    }
}

pub struct TuringMachine {
    tape: Tape,
    initial_tape: Tape,
    initial_state: State,
    current_state: State,
    transitions: HashMap<ReadingState, Vec<Transition>>,
    history: Vec<(Tape, State)>,
    accept_states: Vec<State>,
    machine_type: TuringMachineType
}

impl TuringMachine {
    pub fn step(&mut self) -> Option<HaltState> {
        if matches!(self.machine_type, TuringMachineType::NonDeterministic(_)) {
            todo!("Implement non-deterministic Turing machine stepping logic");
        }

        if self.history.len() >= self.tape.size {
            return Some(HaltState::Reject(RejectReason::Timeout));
        }

        if self.tape.read().len() > self.tape.size {
            return Some(HaltState::Reject(RejectReason::TapeOverflow));
        }

        let reading_state = (self.current_state, self.tape.head);

        if self.accept_states.contains(&self.current_state) {
            return Some(HaltState::Accept);
        }

        if let Some(&(new_state, new_symbol, direction)) = self.transitions.get(&reading_state).and_then(|v| v.first()) {
            self.history.push((self.tape.clone(), self.current_state));

            self.tape.write(new_symbol);
            self.current_state = new_state;

            match direction {
                Direction::Left => {
                    if let Some(symbol) = self.tape.left.pop() {
                        self.tape.right.push(self.tape.head);
                        self.tape.write(symbol);
                    } else {
                        match self.machine_type {
                            TuringMachineType::Deterministic(Characteristics { tape_type: TapeType::SemiInfinite(TapeBoundary::Left), .. }) |
                            TuringMachineType::NonDeterministic(Characteristics { tape_type: TapeType::SemiInfinite(TapeBoundary::Left), .. }) => {
                                return Some(HaltState::Reject(RejectReason::HitWall));
                            },
                            _ => {
                                self.tape.right.push(self.tape.head);
                                self.tape.write(None);
                            }
                        }
                    }
                },
                Direction::Right => {
                    if let Some(symbol) = self.tape.right.pop() {
                        self.tape.left.push(self.tape.head);
                        self.tape.write(symbol);
                    } else {
                        self.tape.left.push(self.tape.head);
                        self.tape.write(None);
                    }
                },
                Direction::Stay => {
                    match self.machine_type {
                        TuringMachineType::Deterministic(Characteristics { move_type: MoveType::Strict, .. }) |
                        TuringMachineType::NonDeterministic(Characteristics { move_type: MoveType::Strict, .. }) => {
                            return Some(HaltState::Reject(RejectReason::InvalidTransition));
                        },
                        _ => {}
                    }
                }
            }

            None
        } else {
            return Some(HaltState::Reject(RejectReason::NoTransition));
        }
    }

    pub fn reset(&mut self) {
        self.tape = self.initial_tape.clone();
        self.current_state = self.initial_state;
        self.history.clear();
    }

    pub fn back(&mut self) -> bool {
        if let Some((tape, state)) = self.history.pop() {
            self.tape = tape;
            self.current_state = state;

            true
        } else {
            false
        }
    }

    pub fn tape(&self) -> &Tape {
        &self.tape
    }

    pub fn current_state(&self) -> State {
        self.current_state
    }

    pub fn initial_state(&self) -> State {
        self.initial_state
    }

    pub fn transitions(&self) -> &HashMap<ReadingState, Vec<Transition>> {
        &self.transitions
    }

    pub fn history(&self) -> &Vec<(Tape, State)> {
        &self.history
    }

    pub fn accept_states(&self) -> &Vec<State> {
        &self.accept_states
    }

    pub fn machine_type(&self) -> &TuringMachineType {
        &self.machine_type
    }
}

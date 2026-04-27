use crate::halt::HaltingState;

pub mod deterministic;
pub mod non_deterministic;

pub trait Computable {
    /// Runs the Turing machine until it halts, returning the halting state.
    fn run(&mut self) -> HaltingState {
        loop {
            if let Some(halt_state) = self.run_once() {
                return halt_state;
            }
        }
    }

    fn run_once(&mut self) -> Option<HaltingState>;
    fn reset(&mut self);
    fn back(&mut self);
}

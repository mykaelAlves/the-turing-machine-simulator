use crate::halt::HaltingState;

pub mod headd;
pub mod headn;
pub mod multid;
pub mod multin;
pub mod singled;
pub mod singlen;

pub trait Computable {
    fn run(&mut self) -> HaltingState;
    fn run_once(&mut self) -> Option<HaltingState>;
    fn reset(&mut self);
    fn back(&mut self);
}

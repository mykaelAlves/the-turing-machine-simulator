use crate::{tape::single::SingleTape, types::Symbol};

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

use std::collections::VecDeque;

use crate::types::Symbol;

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

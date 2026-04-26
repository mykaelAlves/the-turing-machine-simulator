use crate::types::Symbol;

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

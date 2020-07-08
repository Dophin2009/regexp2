mod table;

pub mod dfa;
pub mod nfa;

pub use dfa::{DFAIterator, DFA};
pub use nfa::{NFAIterator, NFA};

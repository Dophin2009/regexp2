#![deny(rust_2018_idioms)]
#![deny(future_incompatible)]

mod matching;

pub mod convert;
pub mod dfa;
pub mod nfa;
pub mod table;

pub use dfa::DFA;
pub use matching::Match;
pub use nfa::NFA;

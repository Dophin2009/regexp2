#![feature(iterator_fold_self)]

mod class;
mod parser;
mod ranges;
mod regexp;
mod table;
mod util;

pub mod nfa;

pub use regexp::*;

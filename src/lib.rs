#![feature(iterator_fold_self)]

mod ast;
mod class;
mod disjoint;
mod parser;
mod ranges;
mod regexp;
mod table;
mod util;

pub mod dfa;
pub mod nfa;

pub use regexp::*;

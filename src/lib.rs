#![feature(iterator_fold_self)]

mod regexp;

mod ast;
mod disjoint;
mod ranges;
mod table;

pub mod class;
pub mod dfa;
pub mod engines;
pub mod nfa;
pub mod parser;

pub use regexp::*;

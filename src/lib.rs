#![feature(iterator_fold_self)]

mod regexp;

mod ast;
mod disjoint;
mod ranges;

pub mod class;
pub mod parser;

pub use automata;
pub use regexp::*;

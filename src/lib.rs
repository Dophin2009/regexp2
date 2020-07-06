#![feature(iterator_fold_self)]

mod class;
mod parser;
mod ranges;
mod regexp;
mod table;
mod util;

#[cfg(test)]
mod tests;

pub mod nfa;

pub use regexp::*;

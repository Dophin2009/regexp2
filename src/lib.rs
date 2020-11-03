#![feature(iterator_fold_self)]
#![feature(type_alias_impl_trait)]

mod regexp;

mod ast;
mod disjoint;
mod ranges;

pub mod class;
pub mod parser;

pub use automata;
pub use regexp::*;

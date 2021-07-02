//! ```
//! use regexp2::RegExp;
//!
//! // Any sequence of a's and b's ending in abb.
//! let mut re = RegExp::new("(a|b)*abb").unwrap();
//! assert!(re.is_match("abb"));
//! assert!(re.is_match("aababb"));
//!
//! // Any sequence of characters that are not B, C, D, E, F or any lowercase
//! // letter.
//! re = RegExp::new("[^B-Fa-z]*").unwrap();
//! assert!(re.is_match("AGAQR"));
//!
//! // Any sequence of at least one digit followed by nothing or an
//! // alphanumeric or underscore.
//! re = RegExp::new(r"\d+\w?").unwrap();
//! assert!(re.is_match("3a"));
//! assert!(re.is_match("08m"));
//! assert!(re.is_match("999_"));
//! ```

#![deny(rust_2018_idioms)]
#![deny(future_incompatible)]

mod regexp;

mod ast;
mod mergeset;
mod ranges;

pub mod class;
pub mod parser;

pub use automata;
pub use regexp::*;

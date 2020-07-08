use crate::class::CharClass;
use crate::dfa::DFA;
use crate::engines::{Engine, NFAParser};
use crate::nfa::NFA;
use crate::parser::{self, Parser};

/// A compiled regular expression for matching strings. It may be used to determine if given
/// strings are within the language described by the regular expression.
#[derive(Debug)]
pub struct RegExp<E: Engine> {
    /// The regular expression represented by this structure.
    expr: String,
    /// The compiled backend of the regular expression used to evaluate input strings.
    engine: E,
}

impl<E: Engine> RegExp<E> {
    /// Determine if the given input string is within the language described by the regular
    /// expression.
    pub fn is_exact_match(&self, input: &str) -> bool {
        self.engine.is_exact_match(input)
    }
}

impl RegExp<NFA<CharClass>> {
    /// Create a compiled regular expression that uses an NFA to evaluate input strings.
    pub fn new(expr: &str) -> parser::Result<Self> {
        let parser = NFAParser::new();
        let nfa: NFA<CharClass> = parser.parse(expr)?.unwrap();

        Ok(RegExp {
            expr: expr.to_owned(),
            engine: nfa,
        })
    }
}

impl RegExp<DFA<CharClass>> {
    /// Create a compiled regular expression that uses a DFA to evaluate input strings.
    pub fn new_with_dfa(expr: &str) -> parser::Result<Self> {
        let parser = NFAParser::new();
        let nfa: NFA<CharClass> = parser.parse(expr)?.unwrap();
        let dfa = nfa.into();

        Ok(RegExp {
            expr: expr.to_owned(),
            engine: dfa,
        })
    }
}

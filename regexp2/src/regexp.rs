use crate::class::CharClass;
use crate::parser::{self, nfa::NFAParser};

use std::ops::Range;

use automata::{self, nfa::Transition, DFA, NFA};

pub use parser::ParseResult;

#[derive(Debug)]
pub struct Match {
    start: usize,
    end: usize,

    pub span: String,
}

impl Match {
    #[inline]
    pub const fn new(start: usize, end: usize, span: String) -> Self {
        Self { start, end, span }
    }

    #[inline]
    pub const fn start(&self) -> usize {
        self.start
    }

    #[inline]
    pub const fn end(&self) -> usize {
        self.end
    }

    #[inline]
    pub const fn range(&self) -> Range<usize> {
        self.start..self.end
    }
}

impl From<automata::Match<char>> for Match {
    #[inline]
    fn from(m: automata::Match<char>) -> Self {
        Self::new(m.start(), m.end(), m.span.into_iter().collect())
    }
}

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
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.expr
    }

    /// Determine if the given input string is within the language described by the regular
    /// expression.
    #[inline]
    pub fn is_match(&self, input: &str) -> bool {
        self.engine.is_match(input)
    }

    #[inline]
    pub fn find(&self, input: &str) -> Option<Match> {
        self.find_at(input, 0)
    }

    #[inline]
    pub fn find_at(&self, input: &str, start: usize) -> Option<Match> {
        self.engine.find_at(input, start)
    }

    #[inline]
    pub fn find_shortest(&self, input: &str) -> Option<Match> {
        self.find_shortest_at(input, 0)
    }

    #[inline]
    pub fn find_shortest_at(&self, input: &str, start: usize) -> Option<Match> {
        self.engine.find_shortest_at(input, start)
    }
}

impl RegExp<NFA<CharClass>> {
    /// Create a compiled regular expression that uses an NFA to evaluate input strings.
    #[inline]
    pub fn new_nfa(expr: &'_ str) -> ParseResult<'_, Self> {
        let parser = NFAParser::new();
        let nfa: NFA<CharClass> = parser.parse(expr)?;

        Ok(RegExp {
            expr: expr.to_owned(),
            engine: nfa,
        })
    }

    #[inline]
    pub fn with_dfa(self) -> RegExp<DFA<CharClass>> {
        RegExp {
            expr: self.expr,
            engine: self.engine.into(),
        }
    }
}

impl RegExp<DFA<CharClass>> {
    /// Create a compiled regular expression that uses a DFA to evaluate input strings.
    #[inline]
    pub fn new(expr: &'_ str) -> ParseResult<'_, Self> {
        Ok(RegExp::new_nfa(expr)?.with_dfa())
    }
}

impl PartialEq<char> for CharClass {
    #[inline]
    fn eq(&self, other: &char) -> bool {
        self.contains(*other)
    }
}

impl From<CharClass> for Transition<CharClass> {
    #[inline]
    fn from(c: CharClass) -> Self {
        Transition::Some(c)
    }
}

/// A trait implemented by regular expression backends, used to evaluate input strings.
pub trait Engine {
    fn is_match(&self, input: &str) -> bool;

    fn find_at(&self, input: &str, start: usize) -> Option<Match>;

    fn find_shortest_at(&self, input: &str, start: usize) -> Option<Match>;
}

impl Engine for NFA<CharClass> {
    #[inline]
    fn is_match(&self, input: &str) -> bool {
        NFA::is_match(self, input.chars())
    }

    #[inline]
    fn find_shortest_at(&self, input: &str, start: usize) -> Option<Match> {
        NFA::find_shortest_at(self, input.chars(), start).map(From::from)
    }

    #[inline]
    fn find_at(&self, input: &str, start: usize) -> Option<Match> {
        NFA::find_at(self, input.chars(), start).map(From::from)
    }
}

impl Engine for DFA<CharClass> {
    #[inline]
    fn is_match(&self, input: &str) -> bool {
        DFA::is_match(self, input.chars())
    }

    #[inline]
    fn find_shortest_at(&self, input: &str, start: usize) -> Option<Match> {
        DFA::find_shortest_at(self, input.chars(), start).map(From::from)
    }

    #[inline]
    fn find_at(&self, input: &str, start: usize) -> Option<Match> {
        DFA::find_at(self, input.chars(), start).map(From::from)
    }
}

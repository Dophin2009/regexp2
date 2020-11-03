use crate::class::{CharClass, CharRange};
use crate::parser::{self, NFAParser, Parser};

use std::convert::TryInto;

pub use automata::Match;
use automata::{dfa::Disjoin, nfa::Transition, DFA, NFA};

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
    pub fn is_match(&self, input: &str) -> bool {
        self.engine.is_match(input)
    }

    pub fn has_match(&self, input: &str) -> bool {
        self.has_match_at(input, 0)
    }

    pub fn has_match_at(&self, input: &str, start: usize) -> bool {
        self.find_shortest_at(input, start).is_some()
    }

    pub fn find(&self, input: &str) -> Option<Match> {
        self.find_at(input, 0)
    }

    pub fn find_at(&self, input: &str, start: usize) -> Option<Match> {
        self.engine.find_at(input, start)
    }

    pub fn find_shortest(&self, input: &str) -> Option<Match> {
        self.find_shortest_at(input, 0)
    }

    pub fn find_shortest_at(&self, input: &str, start: usize) -> Option<Match> {
        self.engine.find_shortest_at(input, start)
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

impl PartialEq<char> for CharClass {
    fn eq(&self, other: &char) -> bool {
        self.contains(*other)
    }
}

impl From<CharClass> for Transition<CharClass> {
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
    fn is_match(&self, input: &str) -> bool {
        NFA::is_match(self, &input.chars())
    }

    fn find_shortest_at(&self, input: &str, start: usize) -> Option<Match> {
        NFA::find_shortest_at(self, &input.chars(), start)
    }

    fn find_at(&self, input: &str, start: usize) -> Option<Match> {
        NFA::find_at(self, &input.chars(), start)
    }
}

impl Engine for DFA<CharClass> {
    fn is_match(&self, input: &str) -> bool {
        DFA::is_match(self, &input.chars())
    }

    fn find_shortest_at(&self, input: &str, start: usize) -> Option<Match> {
        DFA::find_shortest_at(self, &input.chars(), start).map(|(m, _)| m)
    }

    fn find_at(&self, input: &str, start: usize) -> Option<Match> {
        DFA::find_at(self, &input.chars(), start).map(|(m, _)| m)
    }
}

impl Disjoin for CharClass {
    /// Create a set of disjoint CharClass from a set of CharClass. Algorithm inspired by [this
    /// Stack Overflow answer](https://stackoverflow.com/a/55482655/8955108).
    fn disjoin(vec: Vec<&Self>) -> Vec<Self> {
        let ranges: Vec<_> = vec.iter().flat_map(|cc| cc.ranges.clone()).collect();

        let mut starts: Vec<_> = ranges.iter().map(|r| (r.start as u32, 1)).collect();
        let mut ends: Vec<_> = ranges.iter().map(|r| (r.end as u32 + 1, -1)).collect();
        starts.append(&mut ends);
        starts.sort_by(|a, b| a.0.cmp(&b.0));

        let mut prev = 0;
        let mut count = 0;
        starts
            .into_iter()
            .filter_map(|(x, c)| {
                let ret = if x > prev && count != 0 {
                    let ret = CharRange::new(prev.try_into().unwrap(), (x - 1).try_into().unwrap());
                    Some(ret.into())
                } else {
                    None
                };
                prev = x;
                count += c;
                ret
            })
            .collect()
    }

    fn contains(&self, other: &Self) -> bool {
        !self.intersection(other).is_empty()
    }
}

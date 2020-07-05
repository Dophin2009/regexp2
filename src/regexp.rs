use crate::nfa::{Transition, NFA};
use crate::parser::{self, Operator, ParseError, Parser};
use std::hash::Hash;
use std::marker::PhantomData;

// Use the regex-syntax crate to convert ranges of Unicode scalar values to equivalent sets of
// ranges of Unicode codepoints.
// use regex_syntax::utf8::{Utf8Sequence, Utf8Sequences};

#[derive(Debug)]
pub struct RegExp<B: RegExpBackend> {
    expr: String,
    engine: B,
}

impl<B: RegExpBackend> RegExp<B> {
    pub fn is_exact_match(&self, input: &str) -> bool {
        self.engine.is_exact_match(input)
    }
}

impl RegExp<NFA<char>> {
    pub fn new_with_nfa(expr: &str) -> parser::Result<Self> {
        let parser = NFAParser::new();
        let nfa: NFA<char> = parser.parse(expr)?.unwrap();

        Ok(RegExp {
            expr: expr.to_owned(),
            engine: nfa,
        })
    }
}

pub trait RegExpBackend {
    fn is_exact_match(&self, input: &str) -> bool;
}

impl RegExpBackend for NFA<char> {
    fn is_exact_match(&self, input: &str) -> bool {
        self.is_exact_match(input.chars())
    }
}

impl From<char> for Transition<char> {
    fn from(c: char) -> Self {
        Transition::Some(c)
    }
}

pub struct NFAParser<T>
where
    T: Clone + Eq + Hash,
    Transition<T>: From<char>,
{
    _phantom: PhantomData<T>,
}

impl<T> NFAParser<T>
where
    T: Clone + Eq + Hash,
    Transition<T>: From<char>,
{
    pub fn new() -> Self {
        NFAParser {
            _phantom: PhantomData,
        }
    }
}

impl<T> Parser<NFA<T>> for NFAParser<T>
where
    T: Clone + Eq + Hash,
    Transition<T>: From<char>,
{
    fn shift_action(
        &self,
        stack: &mut Vec<NFA<T>>,
        _: &mut Vec<Operator>,
        c: char,
    ) -> parser::Result<()> {
        let transition = c.into();

        let mut nfa = NFA::new();
        let final_state = nfa.add_state(true);
        nfa.add_transition(nfa.initial_state, final_state, transition);

        stack.push(nfa);

        Ok(())
    }

    fn reduce_action(
        &self,
        stack: &mut Vec<NFA<T>>,
        op_stack: &mut Vec<Operator>,
    ) -> parser::Result<()> {
        let op = op_stack.pop().ok_or(ParseError::UnbalancedOperators)?;
        let mut new_nfa: NFA<T>;

        match op {
            Operator::Union => {
                let c2 = stack.pop().ok_or(ParseError::UnbalancedOperators)?;
                let c1 = stack.pop().ok_or(ParseError::UnbalancedOperators)?;
                new_nfa = NFA::union(&c1, &c2);
            }
            Operator::Concatenation => {
                let c2 = stack.pop().ok_or(ParseError::UnbalancedOperators)?;
                let c1 = stack.pop().ok_or(ParseError::UnbalancedOperators)?;
                new_nfa = NFA::concatenation(&c1, &c2);
            }
            Operator::KleeneStar => {
                let c1 = stack.pop().ok_or(ParseError::UnbalancedOperators)?;
                new_nfa = NFA::kleene_star(&c1);
            }
            Operator::EmptyPlaceholder => {
                new_nfa = NFA::new();
                new_nfa.final_states.insert(new_nfa.initial_state);
            }
            Operator::LeftParen => return Err(ParseError::UnbalancedParentheses),
        }

        stack.push(new_nfa);
        Ok(())
    }
}

use crate::class::CharClass;
use crate::nfa::{Transition, NFA};
use crate::parser::{self, Operator, ParseError, Parser};
use std::hash::Hash;
use std::marker::PhantomData;

impl RegExp<NFA<CharClass>> {
    /// Create a compiled regular expression that uses an NFA to evaluate input strings.
    pub fn new_with_nfa(expr: &str) -> parser::Result<Self> {
        let parser = NFAParser::new();
        let nfa: NFA<CharClass> = parser.parse(expr)?.unwrap();

        Ok(RegExp {
            expr: expr.to_owned(),
            engine: nfa,
        })
    }
}

impl Engine for NFA<CharClass> {
    fn is_exact_match(&self, input: &str) -> bool {
        self.is_exact_match(input.chars())
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

/// A trait implemented by regular expression backends, used to evaluate input strings.
pub trait Engine {
    fn is_exact_match(&self, input: &str) -> bool;
}

/// A regular expression parser that produces an NFA that describes the same language as the
/// regular expression. The transitions of the NFA must be derivable from CharClass.
pub struct NFAParser<T>
where
    T: Clone + Eq + Hash,
    Transition<T>: From<CharClass>,
{
    _phantom: PhantomData<T>,
}

impl<T> NFAParser<T>
where
    T: Clone + Eq + Hash,
    Transition<T>: From<CharClass>,
{
    /// Create a new NFAParser.
    pub fn new() -> Self {
        NFAParser {
            _phantom: PhantomData,
        }
    }
}

impl<T> Parser<NFA<T>> for NFAParser<T>
where
    T: Clone + Eq + Hash,
    Transition<T>: From<CharClass>,
{
    /// Implement the shift action. A new NFA with two states and a single transition on the given
    /// character between them is pushed to the parsing stack.
    fn shift_action(
        &self,
        stack: &mut Vec<NFA<T>>,
        _: &mut Vec<Operator>,
        c: CharClass,
    ) -> parser::Result<()> {
        let transition = c.into();

        let mut nfa = NFA::new();
        let final_state = nfa.add_state(true);
        nfa.add_transition(nfa.initial_state, final_state, transition);

        stack.push(nfa);

        Ok(())
    }

    /// Implement the reduce action for parsing. The most recent operator is popped from the stack
    /// and sub-NFAs are popped from the NFA stack, and a new NFA is constructed and pushed to the
    /// stack.
    fn reduce_action(
        &self,
        stack: &mut Vec<NFA<T>>,
        op_stack: &mut Vec<Operator>,
    ) -> parser::Result<()> {
        // Pop the last operator off.
        let op = op_stack.pop().ok_or(ParseError::UnbalancedOperators)?;
        let mut new_nfa: NFA<T>;

        match op {
            // A union NFA is constructed from the 2 operands of the union operator.
            Operator::Union => {
                let c2 = stack.pop().ok_or(ParseError::UnbalancedOperators)?;
                let c1 = stack.pop().ok_or(ParseError::UnbalancedOperators)?;
                new_nfa = NFA::union(&c1, &c2);
            }
            // A concatenated NFA is constructed from the 2 operands of the concatenation
            // operator.
            Operator::Concatenation => {
                let c2 = stack.pop().ok_or(ParseError::UnbalancedOperators)?;
                let c1 = stack.pop().ok_or(ParseError::UnbalancedOperators)?;
                new_nfa = NFA::concatenation(&c1, &c2);
            }
            // A new NFA is constructed from the most recent NFA on the stack for kleene star,
            // plus, and optional operators.
            Operator::KleeneStar => {
                let c1 = stack.pop().ok_or(ParseError::UnbalancedOperators)?;
                new_nfa = NFA::kleene_star(&c1);
            }
            Operator::Plus => {
                let c1 = stack.pop().ok_or(ParseError::UnbalancedOperators)?;
                let kleene = NFA::kleene_star(&c1);
                new_nfa = NFA::concatenation(&kleene, &c1);
            }
            Operator::Optional => {
                let c1 = stack.pop().ok_or(ParseError::UnbalancedOperators)?;
                let c2 = NFA::new_epsilon();
                new_nfa = NFA::union(&c1, &c2);
            }
            // A new NFA with a single epsilon transition is pushed to the stack.
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

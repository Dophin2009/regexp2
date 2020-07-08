use crate::ast::{self, ASTNode};
use crate::class::{CharClass, CharRange};
use crate::dfa::{Disjoin, DFA};
use crate::nfa::{Transition, NFA};
use crate::parser::{self, Operator, ParseError, Parser};
use std::convert::{TryFrom, TryInto};
use std::hash::Hash;
use std::marker::PhantomData;

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

impl Engine for DFA<CharClass> {
    fn is_exact_match(&self, input: &str) -> bool {
        self.is_exact_match(input.chars())
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

pub struct ASTParser<T>
where
    T: Clone + Eq + Hash + From<CharClass>,
{
    _phantom: PhantomData<T>,
}

impl<T> ASTParser<T>
where
    T: Clone + Eq + Hash + From<CharClass>,
{
    /// Create a new ASTParser.
    pub fn new() -> Self {
        ASTParser {
            _phantom: PhantomData,
        }
    }
}

impl<T> Parser<ASTNode<T>> for ASTParser<T>
where
    T: Clone + Eq + Hash + From<CharClass>,
{
    /// Implement the shift action. A new leaf node is pushed to the parsing stack.
    fn shift_action(
        &self,
        stack: &mut Vec<ASTNode<T>>,
        _: &mut Vec<Operator>,
        c: CharClass,
    ) -> parser::Result<()> {
        let new_node = ASTNode::Leaf(c.into());
        stack.push(new_node);
        Ok(())
    }

    /// Implement the reduce action for parsing. The most recent operator is popped from the stack
    /// and child nodes are popped from the node stack, and a new node is constructed and pushed to
    /// the stack.
    fn reduce_action(
        &self,
        stack: &mut Vec<ASTNode<T>>,
        op_stack: &mut Vec<Operator>,
    ) -> parser::Result<()> {
        // Pop the last operator off.
        let op = op_stack.pop().ok_or(ParseError::UnbalancedOperators)?;

        let new_node;
        if op == Operator::EmptyPlaceholder {
            // A new blank leaf node is pushed to the stack if operator is an empty placeholder.
            new_node = ASTNode::None;
        } else {
            // Otherwise, a new branch node is constructed from operands.
            let node_op = op
                .try_into()
                .map_err(|_| ParseError::UnbalancedParentheses)?;
            let c1: ASTNode<T>;
            let c2: ASTNode<T>;

            match node_op {
                // Union and concatenation branch nodes are constructed from the 2 topmost nodes.
                ast::Operator::Union | ast::Operator::Concatenation => {
                    c2 = stack.pop().ok_or(ParseError::UnbalancedOperators)?;
                    c1 = stack.pop().ok_or(ParseError::UnbalancedOperators)?;
                }
                // A new node is constructed from the topmost node on the stack for kleene star,
                // plus, and optional operators.
                ast::Operator::KleeneStar | ast::Operator::Plus | ast::Operator::Optional => {
                    c1 = stack.pop().ok_or(ParseError::UnbalancedOperators)?;
                    c2 = ASTNode::None;
                }
            }

            new_node = ASTNode::Branch(node_op, Box::new(c1), Box::new(c2));
        }

        stack.push(new_node);
        Ok(())
    }
}

impl TryFrom<Operator> for ast::Operator {
    type Error = ();

    fn try_from(op: Operator) -> Result<Self, Self::Error> {
        match op {
            Operator::KleeneStar => Ok(Self::KleeneStar),
            Operator::Plus => Ok(Self::Plus),
            Operator::Optional => Ok(Self::Optional),
            Operator::Concatenation => Ok(Self::Concatenation),
            Operator::Union => Ok(Self::Union),
            Operator::EmptyPlaceholder => Err(()),
            Operator::LeftParen => Err(()),
        }
    }
}

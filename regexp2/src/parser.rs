use crate::class::CharClass;

use std::hash::Hash;
use std::iter::Peekable;
use std::marker::PhantomData;
use std::str::CharIndices;

use automata::nfa::Transition;
use automata::NFA;

/// Alias for [`Result`] for [`ParseError`].
pub type ParseResult<'r, T> = std::result::Result<T, ParseError<'r>>;

#[derive(Debug)]
pub struct Parser<E>
where
    E: ParserEngine,
{
    _phantom: PhantomData<E>,
}

impl<E> Parser<E>
where
    E: ParserEngine,
{
    #[inline]
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    #[inline]
    pub fn parse<'r>(&self, expr: &'r str) -> ParseResult<'r, E::Output> {
        let mut state: ParserState<E> = ParserState::new();
        state.parse(expr)
    }
}

#[derive(Debug)]
pub struct ParserState<E>
where
    E: ParserEngine,
{
    engine: E,
}

pub trait ParserEngine {
    type Output;

    fn new() -> Self;

    fn handle_char(&mut self, c: char) -> Self::Output;
}

impl<E> ParserState<E>
where
    E: ParserEngine,
{
    #[inline]
    pub fn new() -> Self {
        Self { engine: E::new() }
    }

    /// Compile a regular expresion.
    #[inline]
    pub fn parse<'r>(&mut self, expr: &'r str) -> ParseResult<'r, E::Output> {
        let input = &mut ParseInput::new(expr);
        self.parse_expr(input)
    }

    #[inline]
    fn parse_expr<'r>(&mut self, input: &mut ParseInput<'r>) -> ParseResult<'r, E::Output> {
        match input.peek() {
            Some((_, c)) => match c {
                '(' => self.parse_group(input),
                // '[' => self.parse_class(input)?,
                _ => {
                    let (_, c) = input.next().unwrap();
                    Ok(self.engine.handle_char(c))
                }
            },
            None => Err(ParseError::EmptyExpression {
                span: input.current_span(),
            }),
        }
    }

    #[inline]
    fn parse_group<'r>(&mut self, input: &mut ParseInput<'r>) -> ParseResult<'r, E::Output> {
        let _lparen = input.next();
        let expr = self.parse_expr(input)?;
        let _rparen = input.next();

        Ok(expr)
    }
}

struct ParseInput<'r> {
    expr: &'r str,
    input: Peekable<CharIndices<'r>>,

    next_pos: usize,
    char_pos: usize,
}

impl<'r> ParseInput<'r> {
    #[inline]
    pub fn new(expr: &'r str) -> Self {
        Self {
            expr,
            input: expr.char_indices().peekable(),
            next_pos: 0,
            char_pos: 0,
        }
    }

    #[inline]
    pub fn next(&mut self) -> Option<(usize, char)> {
        let next = self.input.next();
        if let Some((char_pos, _)) = next {
            self.next_pos += 1;
            self.char_pos = char_pos;
        }

        next
    }

    #[inline]
    pub fn next_unwrap<F>(&mut self, expected: F) -> ParseResult<'r, (usize, char)>
    where
        F: Fn() -> Vec<char>,
    {
        match self.next() {
            Some(c) => Ok(c),
            None => Err(ParseError::UnexpectedEof {
                span: self.current_eof_span(),
                expected: expected(),
            }),
        }
    }

    #[inline]
    pub fn next_checked<F>(&mut self, check: char, expected: F) -> ParseResult<'r, (usize, char)>
    where
        F: Fn() -> Vec<char>,
    {
        match self.next() {
            Some(next) if next.1 == check => Ok(next),
            Some(next) => Err(ParseError::UnexpectedToken {
                span: self.current_span(),
                token: next.1,
                expected: expected(),
            }),
            None => Err(ParseError::UnexpectedEof {
                span: self.current_eof_span(),
                expected: expected(),
            }),
        }
    }

    #[inline]
    pub fn peek(&mut self) -> Option<&(usize, char)> {
        self.input.peek()
    }

    #[inline]
    pub fn peek_is(&mut self, expected: char) -> bool {
        match self.peek() {
            Some(peeked) => peeked.1 == expected,
            None => false,
        }
    }

    #[inline]
    pub fn is_empty(&mut self) -> bool {
        self.input.peek().is_none()
    }

    #[inline]
    pub fn expr(&self) -> &str {
        self.expr
    }

    #[inline]
    fn current_span(&mut self) -> Span<'r> {
        let pos = if self.next_pos == 0 {
            0
        } else {
            self.next_pos - 1
        };

        let text = match self.input.peek() {
            Some((end, _)) => &self.expr[self.char_pos..*end],
            None => &self.expr[self.char_pos..],
        };

        Span::new(pos, pos, text)
    }

    #[inline]
    fn current_eof_span(&self) -> Span<'r> {
        let pos = self.next_pos;
        Span::new(pos, pos, "")
    }
}

/// Error returned when attempting to parse an invalid regular expression.
#[derive(Debug, thiserror::Error)]
pub enum ParseError<'r> {
    #[error("empty regular expression")]
    EmptyExpression { span: Span<'r> },

    #[error("unexpected token")]
    UnexpectedToken {
        span: Span<'r>,
        token: char,
        expected: Vec<char>,
    },
    #[error("unexpected end-of-file")]
    UnexpectedEof { span: Span<'r>, expected: Vec<char> },

    /// There are an invalid number of operators, or operands are missing.
    #[error("unbalanced operators")]
    UnbalancedOperators { span: Span<'r> },
    /// There are one or more sets of unclosed parentheses.
    #[error("unbalanced operators")]
    UnbalancedParentheses { span: Span<'r> },
    /// Bracketed character classes may not empty.
    #[error("empty character class")]
    EmptyCharacterClass { span: Span<'r> },
}

#[derive(Debug)]
pub struct Span<'r> {
    start: usize,
    end: usize,

    text: &'r str,
}

impl<'r> Span<'r> {
    #[inline]
    pub fn new(start: usize, end: usize, text: &'r str) -> Self {
        Self { start, end, text }
    }

    #[inline]
    pub fn start(&self) -> usize {
        self.start
    }

    #[inline]
    pub fn end(&self) -> usize {
        self.end
    }

    #[inline]
    pub fn text(&self) -> &str {
        self.text
    }
}

pub type NFAParser<T> = Parser<NFAParserEngine<T>>;

/// A regular expression parser that produces an NFA that describes the same language as the
/// regular expression. The transitions of the NFA must be derivable from CharClass.
pub struct NFAParserEngine<T>
where
    T: Clone + Eq + Hash,
    Transition<T>: From<CharClass>,
{
    _phantom: PhantomData<T>,
}

impl<T> NFAParserEngine<T>
where
    T: Clone + Eq + Hash,
    Transition<T>: From<CharClass>,
{
    /// Create a new NFAParser.
    #[inline]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        NFAParserEngine {
            _phantom: PhantomData,
        }
    }
}

impl<T> ParserEngine for NFAParserEngine<T>
where
    T: Clone + Eq + Hash,
    Transition<T>: From<CharClass>,
{
    type Output = NFA<T>;

    #[inline]
    fn new() -> Self {
        Self::new()
    }

    #[inline]
    fn handle_char(&mut self, c: char) -> Self::Output {
        let class: CharClass = c.into();
        let transition = class.into();

        let mut nfa = NFA::new();
        let f = nfa.add_state(true);
        nfa.add_transition(nfa.start_state, f, transition);
        nfa
    }
}

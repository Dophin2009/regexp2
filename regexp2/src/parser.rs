use crate::class::{CharClass, CharRange};

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
    #[allow(clippy::new_without_default)]
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

    fn handle_char<C>(&mut self, c: C) -> Self::Output
    where
        C: Into<CharClass>;

    fn handle_wildcard(&mut self) -> Self::Output;
}

impl<E> ParserState<E>
where
    E: ParserEngine,
{
    const EXPR_START_EXPECTED: &'static [char] = &['(', '['];

    #[inline]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { engine: E::new() }
    }

    /// Compile a regular expresion.
    #[inline]
    pub fn parse<'r>(&mut self, expr: &'r str) -> ParseResult<'r, E::Output> {
        let input = &mut ParseInput::new(expr);
        self.parse_expr(input, 0)
    }

    #[inline]
    fn parse_expr<'r>(
        &mut self,
        input: &mut ParseInput<'r>,
        min_bp: usize,
    ) -> ParseResult<'r, E::Output> {
        let mut lhs = None;
        while lhs.is_none() {
            lhs = match input.peek() {
                Some((_, c)) => match c {
                    '\\' => Some(self.parse_escaped(input)?),
                    // Beginning of a group.
                    '(' => self.parse_group(input)?,
                    '[' => self.parse_class(input)?,
                    '.' => Some(self.parse_wildcard(input)?),
                    '*' | '|' => {
                        let (_, c) = input.next_unchecked();
                        return Err(ParseError::UnexpectedToken {
                            span: input.current_span(),
                            token: c,
                            expected: Self::EXPR_START_EXPECTED.into(),
                        });
                    }
                    _ => Some(self.parse_single(input)?),
                },
                None => {
                    return Err(ParseError::EmptyExpression {
                        span: input.current_span(),
                    })
                }
            };
        }

        let lhs = lhs.unwrap();
        // while let Some((_, c)) = input.peek() {
        // lhs = match c {
        // '*' => self.engine.handle_kleene_star(lhs),
        // }
        // }

        Ok(lhs)
    }

    #[inline]
    fn parse_single_char<'r>(&mut self, input: &mut ParseInput<'r>) -> ParseResult<'r, char> {
        // TODO: Expect any
        let (_, c) = input.next_unwrap(Vec::new)?;
        Ok(c)
    }

    #[inline]
    fn parse_single<'r>(&mut self, input: &mut ParseInput<'r>) -> ParseResult<'r, E::Output> {
        let c = self.parse_single_char(input)?;
        Ok(self.engine.handle_char(c))
    }

    #[inline]
    fn parse_escaped_char<'r>(&mut self, input: &mut ParseInput<'r>) -> ParseResult<'r, char> {
        let _bs = input.next_checked('\\', || vec!['\\']);
        // TODO: How to represent expected any character?
        let (_, c) = input.next_unwrap(Vec::new)?;
        Ok(c)
    }

    #[inline]
    fn parse_escaped<'r>(&mut self, input: &mut ParseInput<'r>) -> ParseResult<'r, E::Output> {
        let c = self.parse_escaped_char(input)?;
        Ok(self.engine.handle_char(c))
    }

    #[inline]
    fn parse_single_or_escaped_char<'r>(
        &mut self,
        input: &mut ParseInput<'r>,
    ) -> ParseResult<'r, char> {
        match input.peek() {
            Some((_, '\\')) => self.parse_escaped_char(input),
            Some((_, _)) => self.parse_single_char(input),
            None => Err(ParseError::UnexpectedEof {
                span: input.current_eof_span(),
                expected: vec!['\\'],
            }),
        }
    }

    #[inline]
    fn parse_single_or_escaped<'r>(
        &mut self,
        input: &mut ParseInput<'r>,
    ) -> ParseResult<'r, E::Output> {
        match input.peek() {
            Some((_, '\\')) => self.parse_escaped(input),
            Some((_, _)) => self.parse_single(input),
            None => Err(ParseError::UnexpectedEof {
                span: input.current_eof_span(),
                expected: vec!['\\'],
            }),
        }
    }

    #[inline]
    fn parse_group<'r>(
        &mut self,
        input: &mut ParseInput<'r>,
    ) -> ParseResult<'r, Option<E::Output>> {
        let _lp = input.next_checked('(', || vec!['(']);

        let expr = if !input.peek_is(')') {
            let expr = self.parse_expr(input, 0)?;
            Some(expr)
        } else {
            None
        };

        let _rp = input.next_checked(')', || vec![')']);

        Ok(expr)
    }

    #[inline]
    fn parse_class<'r>(
        &mut self,
        input: &mut ParseInput<'r>,
    ) -> ParseResult<'r, Option<E::Output>> {
        let _lb = input.next_checked('[', || vec!['['])?;

        let negate = match input.peek() {
            Some((_, '^')) => {
                let _caret = input.next_unchecked();
                true
            }
            Some((_, _)) => false,
            None => {
                return Err(ParseError::UnexpectedEof {
                    span: input.current_eof_span(),
                    // TODO: Expect any
                    expected: vec![']', '^'],
                });
            }
        };

        let mut class = CharClass::new();
        while let Some((_, c)) = input.peek() {
            let start = match c {
                // LB indicates end of char class.
                ']' => {
                    let _rb = input.next_checked(']', || vec!['[']);
                    break;
                }
                _ => self.parse_single_or_escaped_char(input)?,
            };

            let end = match input.peek() {
                Some((_, '-')) => {
                    let _dash = input.next_unchecked();
                    self.parse_single_or_escaped_char(input)?
                }
                Some((_, _)) => start,
                None => {
                    return Err(ParseError::UnexpectedEof {
                        span: input.current_eof_span(),
                        // TODO Expect any char
                        expected: vec!['-'],
                    });
                }
            };

            class.add_range(CharRange::new(start, end));
        }

        let _rb = input.next_checked(']', || vec!['[']);

        let v = if !class.is_empty() {
            let class = if negate { class.complement() } else { class };
            Some(self.engine.handle_char(class))
        } else {
            None
        };

        Ok(v)
    }

    #[inline]
    fn parse_wildcard_char<'r>(&mut self, input: &mut ParseInput<'r>) -> ParseResult<'r, char> {
        let (_, c) = input.next_checked('.', || vec!['.'])?;
        Ok(c)
    }

    #[inline]
    fn parse_wildcard<'r>(&mut self, input: &mut ParseInput<'r>) -> ParseResult<'r, E::Output> {
        let _ = self.parse_wildcard_char(input)?;
        Ok(self.engine.handle_wildcard())
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
    pub fn next_unchecked(&mut self) -> (usize, char) {
        self.next().unwrap()
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
    fn handle_char<C>(&mut self, c: C) -> Self::Output
    where
        C: Into<CharClass>,
    {
        let class: CharClass = c.into();
        let transition = class.into();

        let mut nfa = NFA::new();
        let f = nfa.add_state(true);
        nfa.add_transition(nfa.start_state, f, transition);
        nfa
    }

    #[inline]
    fn handle_wildcard(&mut self) -> Self::Output {
        let class = CharClass::all_but_newline();
        self.handle_char(class)
    }
}

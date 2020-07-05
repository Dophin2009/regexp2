use std::error;
use std::fmt;
use std::marker::PhantomData;

#[derive(Debug, PartialEq)]
pub enum Operator {
    Union,
    Concatenation,
    KleeneStar,
    LeftParen,
}

#[derive(Debug)]
pub struct Parser<T, F>
where
    T: Copy,
    F: Copy + FnMut(&ParserState<T, F>, Operator) -> T,
{
    parser_action: F,
    _phantom: PhantomData<T>,
}

impl<T, F> Parser<T, F>
where
    T: Copy,
    F: Copy + FnMut(&ParserState<T, F>, Operator) -> T,
{
    pub fn parse(&self, expr: &str) -> Result<(), ParseError> {
        let mut state = ParserState::new(self.parser_action);

        for c in expr.chars() {
            if state.escaped {
                state.escaped = false;
                state.handle_literal_char(c);
            } else {
                match c {
                    '\\' => state.escaped = true,
                    '|' => state.handle_alter(),
                    '*' => state.handle_kleene_star(),
                    '(' => state.handle_left_paren(),
                    ')' => state.handle_right_paren()?,
                    _ => state.handle_literal_char(c)?,
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct ParserState<T, F>
where
    F: FnMut(&ParserState<T, F>, Operator) -> T,
{
    stack: Vec<T>,
    op_stack: Vec<Operator>,
    paren_count_stack: Vec<usize>,

    escaped: bool,
    insert_concat: bool,

    parser_action: F,
}

impl<T, F> ParserState<T, F>
where
    F: FnMut(&ParserState<T, F>, Operator) -> T,
{
    fn new(parser_action: F) -> Self {
        Self {
            stack: Vec::new(),
            op_stack: Vec::new(),
            paren_count_stack: Vec::new(),

            escaped: false,
            insert_concat: false,

            parser_action,
        }
    }

    fn handle_literal_char(&mut self, c: char) -> Result<(), ParseError> {
        while self.precedence_reduce_stack(&Operator::Concatenation)? {}

        if self.insert_concat {
            self.push_operator(Operator::Concatenation);
        }

        let new = (self.parser_action)(self, );
        self.stack.
    }

    fn handle_alter(&mut self) {
        let op = Operator::Union;
        self.precedence_reduce_stack(&op);

        self.op_stack.push(op);
        self.insert_concat = false;
    }

    fn handle_kleene_star(&mut self) {
        let op = Operator::KleeneStar;
        self.precedence_reduce_stack(&op);

        self.op_stack.push(op);
        self.insert_concat = true;
    }

    fn handle_left_paren(&mut self) {
        let op = Operator::LeftParen;
        self.precedence_reduce_stack(&op);

        if self.insert_concat {
            self.push_concatenation();
        }

        self.op_stack.push(op);
        self.paren_count_stack.push(self.stack.len());
        self.insert_concat = false;
    }

    fn handle_right_paren(&mut self) -> Result<(), ParseError> {
        let last_op = self
            .op_stack
            .last()
            .ok_or(ParseError::UnbalancedOperators)?;
        let prev_node_count = self
            .paren_count_stack
            .last()
            .ok_or(ParseError::UnbalancedParentheses)?;
        if *last_op == Operator::LeftParen && *prev_node_count == self.stack.len() {
            self.op_stack.pop().ok_or(ParseError::UnbalancedOperators)?;
        // self.stack.push()
        } else {
            while !self.op_stack.is_empty() && *self.op_stack.last().unwrap() != Operator::LeftParen
            {
                self.reduce_stack()?;
            }
            self.op_stack.pop().ok_or(ParseError::UnbalancedOperators)?;
        }

        self.insert_concat = true;

        Ok(())
    }

    fn reduce_stack(&mut self) {}

    fn precedence_reduce_stack(&mut self, op: &Operator) -> Result<bool, ParseError> {}

    fn push_operator(&mut self, op: Operator) {
        self.op_stack.push(op);
    }

    fn push_concatenation(&mut self) {
        self.op_stack.push(Operator::Concatenation);
    }
}

#[derive(Debug)]
pub enum ParseError {
    UnbalancedOperators,
    UnbalancedParentheses,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::UnbalancedOperators => write!(f, "unbalanced operators"),
            Self::UnbalancedParentheses => write!(f, "unbalanced parentheses"),
        }
    }
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            _ => None,
        }
    }
}

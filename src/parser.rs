use std::error;
use std::fmt;
use std::result;

pub type Result<T> = result::Result<T, ParseError>;

#[derive(Debug, PartialEq)]
pub enum Operator {
    Union,
    Concatenation,
    KleeneStar,
    LeftParen,
    EmptyPlaceholder,
}

pub trait Parser<T>
where
    T: Clone,
{
    fn shift_action(&self, stack: &mut Vec<T>, op_stack: &mut Vec<Operator>, c: char)
        -> Result<()>;

    fn reduce_action(&self, stack: &mut Vec<T>, op_stack: &mut Vec<Operator>) -> Result<()>;

    fn parse(&self, expr: &str) -> Result<Option<T>> {
        let mut state = ParserState::new(
            |stack, op_stack, c| self.shift_action(stack, op_stack, c),
            |stack, op_stack| self.reduce_action(stack, op_stack),
        );

        for c in expr.chars() {
            if state.escaped {
                state.escaped = false;
                state.handle_literal_char(c)?;
            } else {
                match c {
                    '\\' => state.escaped = true,
                    '|' => state.handle_alter()?,
                    '*' => state.handle_kleene_star()?,
                    '(' => state.handle_left_paren()?,
                    ')' => state.handle_right_paren()?,
                    _ => state.handle_literal_char(c)?,
                }
            }
        }

        if expr.len() == 0 {
            state.op_stack.push(Operator::EmptyPlaceholder);
        }

        while !state.op_stack.is_empty() {
            state.reduce_stack()?;
        }

        let head = state.stack.into_iter().last();
        Ok(head)
    }
}

#[derive(Debug)]
struct ParserState<T, SF, RF>
where
    SF: Copy + FnMut(&mut Vec<T>, &mut Vec<Operator>, char) -> Result<()>,
    RF: Copy + FnMut(&mut Vec<T>, &mut Vec<Operator>) -> Result<()>,
{
    stack: Vec<T>,
    op_stack: Vec<Operator>,
    paren_count_stack: Vec<usize>,

    escaped: bool,
    insert_concat: bool,

    shift_action: SF,
    reduce_action: RF,
}

impl<T, SF, RF> ParserState<T, SF, RF>
where
    SF: Copy + FnMut(&mut Vec<T>, &mut Vec<Operator>, char) -> Result<()>,
    RF: Copy + FnMut(&mut Vec<T>, &mut Vec<Operator>) -> Result<()>,
{
    fn new(shift_action: SF, reduce_action: RF) -> Self {
        Self {
            stack: Vec::new(),
            op_stack: Vec::new(),
            paren_count_stack: Vec::new(),

            escaped: false,
            insert_concat: false,

            shift_action,
            reduce_action,
        }
    }

    fn handle_literal_char(&mut self, c: char) -> Result<()> {
        while self.precedence_reduce_stack(&Operator::Concatenation)? {}

        if self.insert_concat {
            self.push_operator(Operator::Concatenation);
        }

        self.shift_action(c)?;
        self.insert_concat = true;

        Ok(())
    }

    fn handle_alter(&mut self) -> Result<()> {
        let op = Operator::Union;
        self.precedence_reduce_stack(&op)?;

        self.op_stack.push(op);
        self.insert_concat = false;

        Ok(())
    }

    fn handle_kleene_star(&mut self) -> Result<()> {
        let op = Operator::KleeneStar;
        self.precedence_reduce_stack(&op)?;

        self.op_stack.push(op);
        self.insert_concat = true;

        Ok(())
    }

    fn handle_left_paren(&mut self) -> Result<()> {
        let op = Operator::LeftParen;
        self.precedence_reduce_stack(&op);

        if self.insert_concat {
            self.push_concatenation();
        }

        self.op_stack.push(op);
        self.paren_count_stack.push(self.stack.len());
        self.insert_concat = false;

        Ok(())
    }

    fn handle_right_paren(&mut self) -> Result<()> {
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
            self.op_stack.push(Operator::EmptyPlaceholder);
            self.reduce_stack()?;
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

    fn reduce_stack(&mut self) -> Result<()> {
        self.reduce_action()
    }

    fn precedence_reduce_stack(&mut self, op: &Operator) -> Result<bool> {
        let reduce = match self.op_stack.last() {
            Some(last_op) => {
                if last_op == op && *last_op != Operator::LeftParen {
                    // If current op is the same as last, collapse the last.
                    // If both of left parenthesis, do nothing
                    true
                } else if *op == Operator::Union {
                    // If current op is alternation, collapse last if it is kleene or concat.
                    *last_op == Operator::KleeneStar || *last_op == Operator::Concatenation
                } else if *op == Operator::Concatenation {
                    // If current op is concat, collapse last if it is kleene star.
                    *last_op == Operator::KleeneStar
                } else if *op == Operator::KleeneStar {
                    // If current op is kleene star, do not collapse last because kleene star is
                    // highest precedence.
                    false
                } else if *op == Operator::LeftParen {
                    // If current op is left parenthesis, collapse last if it is kleene star.
                    // KleeneStar star operates only on left node.
                    *last_op == Operator::KleeneStar || *last_op == Operator::Concatenation
                } else {
                    false
                }
            }
            None => false,
        };

        if reduce {
            self.reduce_stack()?;
        }

        Ok(reduce)
    }

    fn push_operator(&mut self, op: Operator) {
        self.op_stack.push(op);
    }

    fn push_concatenation(&mut self) {
        self.op_stack.push(Operator::Concatenation);
    }

    fn shift_action(&mut self, c: char) -> Result<()> {
        (self.shift_action)(&mut self.stack, &mut self.op_stack, c)
    }

    fn reduce_action(&mut self) -> Result<()> {
        (self.reduce_action)(&mut self.stack, &mut self.op_stack)
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

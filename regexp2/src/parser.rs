use crate::ast::{self, ASTNode};
use crate::class::{CharClass, CharRange};

use std::convert::{TryFrom, TryInto};
use std::error;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;
use std::result;

use automata::{nfa::Transition, NFA};

/// Alias for [std::result::Result] for [ParseError].
pub type Result<T> = result::Result<T, ParseError>;

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
    #[inline]
    pub fn new() -> Self {
        NFAParser {
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for NFAParser<T>
where
    T: Clone + Eq + Hash,
    Transition<T>: From<CharClass>,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Parser<NFA<T>> for NFAParser<T>
where
    T: Clone + Eq + Hash,
    Transition<T>: From<CharClass>,
{
    /// Implement the shift action. A new NFA with two states and a single transition on the given
    /// character between them is pushed to the parsing stack.
    #[inline]
    fn shift_action(
        &self,
        stack: &mut Vec<NFA<T>>,
        _: &mut Vec<Operator>,
        c: CharClass,
    ) -> Result<()> {
        let transition = c.into();

        let mut nfa = NFA::new();
        let final_state = nfa.add_state(true);
        nfa.add_transition(nfa.start_state, final_state, transition);

        stack.push(nfa);

        Ok(())
    }

    /// Implement the reduce action for parsing. The most recent operator is popped from the stack
    /// and sub-NFAs are popped from the NFA stack, and a new NFA is constructed and pushed to the
    /// stack.
    #[inline]
    fn reduce_action(&self, stack: &mut Vec<NFA<T>>, op_stack: &mut Vec<Operator>) -> Result<()> {
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
                new_nfa.accepting_states.insert(new_nfa.start_state);
            }
            Operator::LeftParen => return Err(ParseError::UnbalancedParentheses),
        }

        stack.push(new_nfa);
        Ok(())
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
    #[inline]
    pub fn new() -> Self {
        ASTParser {
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for ASTParser<T>
where
    T: Clone + Eq + Hash + From<CharClass>,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Parser<ASTNode<T>> for ASTParser<T>
where
    T: Clone + Eq + Hash + From<CharClass>,
{
    /// Implement the shift action. A new leaf node is pushed to the parsing stack.
    #[inline]
    fn shift_action(
        &self,
        stack: &mut Vec<ASTNode<T>>,
        _: &mut Vec<Operator>,
        c: CharClass,
    ) -> Result<()> {
        let new_node = ASTNode::Leaf(c.into());
        stack.push(new_node);
        Ok(())
    }

    /// Implement the reduce action for parsing. The most recent operator is popped from the stack
    /// and child nodes are popped from the node stack, and a new node is constructed and pushed to
    /// the stack.
    #[inline]
    fn reduce_action(
        &self,
        stack: &mut Vec<ASTNode<T>>,
        op_stack: &mut Vec<Operator>,
    ) -> Result<()> {
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

    #[inline]
    fn try_from(op: Operator) -> result::Result<Self, Self::Error> {
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

/// Parser implementations must define the shift and reduce actions. The symbols in a regular
/// expression are iterated through and parsed according to these functions.
pub trait Parser<T>
where
    T: Clone,
{
    fn shift_action(
        &self,
        stack: &mut Vec<T>,
        op_stack: &mut Vec<Operator>,
        c: CharClass,
    ) -> Result<()>;

    fn reduce_action(&self, stack: &mut Vec<T>, op_stack: &mut Vec<Operator>) -> Result<()>;

    /// Compile a regular expresion.
    #[inline]
    fn parse(&self, expr: &str) -> Result<Option<T>> {
        // Overall super spaghetti, needs refactoring and cleaning up.
        let mut state = ParserState::new(
            |stack, op_stack, c| self.shift_action(stack, op_stack, c),
            |stack, op_stack| self.reduce_action(stack, op_stack),
        );

        let mut chars = expr.chars();
        let mut next = chars.next();
        while next.is_some() {
            let c = next.unwrap();

            match c {
                '|' => {
                    if state.escaped {
                        state.escaped = false;
                        if state.in_char_class {
                            // If escaped and in char class, push to char range buffer.
                            state.append_char_range_buf(c);
                        } else {
                            // If escaped and not in char class, handle this as literal |.
                            state.handle_literal_char(c)?;
                        }
                    } else if state.in_char_class {
                        // If not escaped and in char class, push to char range buffer.
                        state.append_char_range_buf(c);
                    } else {
                        // If not escaped and not in char class, handle this as union operator.
                        state.handle_union()?;
                    }
                }
                '*' => {
                    if state.escaped {
                        state.escaped = false;
                        if state.in_char_class {
                            // If escaped and in char class, push to char range buffer.
                            state.append_char_range_buf(c);
                        } else {
                            // If escaped and not in char class, handle this as literal |
                            state.handle_literal_char(c)?;
                        }
                    } else if state.in_char_class {
                        // If not escaped and in char class, push to char range buffer.
                        state.append_char_range_buf(c);
                    } else {
                        // If not escaped, handle this as kleene star operator.
                        state.handle_kleene_star()?;
                    }
                }
                '+' => {
                    if state.escaped {
                        state.escaped = false;
                        if state.in_char_class {
                            // If escaped and in char class, push to char range buffer.
                            state.append_char_range_buf(c);
                        } else {
                            // If escaped and not in char class, handle this as literal +.
                            state.handle_literal_char(c)?;
                        }
                    } else if state.in_char_class {
                        // If not escaped and in char class, push to char range buffer.
                        state.append_char_range_buf(c);
                    } else {
                        // If not escaped and not in char class, handle this as plus operator.
                        state.handle_plus()?;
                    }
                }
                '?' => {
                    if state.escaped {
                        state.escaped = false;
                        if state.in_char_class {
                            // If escaped and in char class, push to char range buffer.
                            state.append_char_range_buf(c);
                        } else {
                            // If escaped and not in char class, handle this as literal ?.
                            state.handle_literal_char(c)?;
                        }
                    } else if state.in_char_class {
                        // If not escaped and in char class, push to char range buffer.
                        state.append_char_range_buf(c);
                    } else {
                        // If not escaped and not in char class, handle this as optional operator.
                        state.handle_optional()?;
                    }
                }
                '(' => {
                    if state.escaped {
                        state.escaped = false;
                        if state.in_char_class {
                            // If escaped and in char class, push to char range buffer.
                            state.append_char_range_buf(c);
                        } else {
                            // If escaped, handle this as literal (.
                            state.handle_literal_char(c)?;
                        }
                    } else if state.in_char_class {
                        // If not escaped and in char class, push to char range buffer.
                        state.append_char_range_buf(c);
                    } else {
                        // If not escaped, handle this as left parentheses
                        state.handle_left_paren()?;
                    }
                }
                ')' => {
                    if state.escaped {
                        state.escaped = false;
                        if state.in_char_class {
                            // If escaped and in char class, push to char range buffer.
                            state.append_char_range_buf(c);
                        } else {
                            // If escaped, handle this as literal |
                            state.handle_literal_char(c)?;
                        }
                    } else if state.in_char_class {
                        // If not escaped and in char class, push to char range buffer.
                        state.append_char_range_buf(c);
                    } else {
                        // If not escaped, handle this as left parentheses
                        state.handle_right_paren()?;
                    }
                }
                '[' => {
                    if state.in_char_class {
                        // Set [ in char class if currently within brackets.
                        state.append_char_range_buf(c);
                    } else if state.escaped {
                        // Handle [ as literal if escaped and not in char class.
                        state.escaped = false;
                        state.handle_literal_char(c)?;
                    } else {
                        // Enter char class until ] is seen if not currently in char class or
                        // escaped.
                        state.in_char_class = true;
                        state.clear_char_class_buf();
                    }
                }
                ']' => {
                    if state.escaped {
                        state.escaped = false;
                        if state.in_char_class {
                            // Handle ] as part in char class if escaped and in char class.
                            state.append_char_range_buf(c);
                        } else {
                            // Handle ] as literal if escaped and not in char class.
                            state.handle_literal_char(c)?;
                        }
                    } else if state.in_char_class {
                        state.handle_right_bracket()?;
                    } else {
                        // Handle ] as literal if not escaped or in char class.
                        state.handle_literal_char(c)?;
                    }
                }
                '\\' => {
                    if state.escaped {
                        // If escaped, handle this as literal \
                        state.escaped = false;
                        state.handle_literal_char(c)?;
                    } else {
                        // If unescaped and in char class, handle next.
                        // If unescaped and not in char class, handle next.
                        state.escaped = true;
                    }
                }
                '^' => {
                    if state.escaped {
                        state.escaped = false;
                        if state.in_char_class {
                            // If escaped and in char class, handle this as literal ^ in char
                            // class.
                            state.append_char_range_buf(c);
                        } else {
                            // If escaped but not in char class, handle this literal ^.
                            state.handle_literal_char(c)?;
                        }
                    } else if state.in_char_class {
                        // If unescaped and in char class, check if this is the first char in the
                        // character class. If so, set flag to negate the current character class
                        // when shifted.
                        if state.char_range_buf.is_empty()
                            && state.char_class_buf.0.ranges.is_empty()
                        {
                            state.char_class_buf.1 = true;
                        } else {
                            // Otherwise push this as regular char to char class.
                            state.append_char_range_buf(c);
                        }
                    } else {
                        // If unescaped and not in char class, handle this as literal ^.
                        state.handle_literal_char(c)?;
                    }
                }
                '.' => {
                    if state.escaped {
                        state.escaped = false;
                        if state.in_char_class {
                            // If escaped and in char class, handle this as a literal . in char class.
                            state.append_char_range_buf(c);
                        } else {
                            // If escaped and not in char class, handle this as a literal .
                            state.handle_literal_char(c)?;
                        }
                    } else if state.in_char_class {
                        // If unescaped and in char class, push . to char range buf as literal.
                        state.append_char_range_buf(c);
                    } else {
                        // If unescaped and not in char class, add ranges for all chars except \n to
                        // char class buf.
                        let cc = CharClass::all_but_newline();
                        state.handle_char_class(cc)?;
                    }
                }
                _ => {
                    // Kinda spaghetti:
                    let mut is_special = true;
                    let mut cc = CharClass::new();
                    if state.escaped {
                        state.escaped = false;
                        // If sequence is \d,
                        if c == 'd' {
                            cc = CharClass::decimal_number();
                        } else if c == 'D' {
                            cc = CharClass::decimal_number().complement();
                        } else if c == 'w' {
                            cc = CharClass::word();
                        } else if c == 'W' {
                            cc = CharClass::word().complement();
                        } else if c == 'n' {
                            cc = '\n'.into();
                        } else if c == 's' {
                            cc = CharClass::whitespace();
                        } else if c == 'S' {
                            cc = CharClass::whitespace().complement();
                        } else {
                            is_special = false;
                        }
                    } else {
                        is_special = false;
                    }

                    if is_special {
                        if state.in_char_class {
                            state.handle_incomplete_char_range_buf();
                            state.char_class_buf.0.copy_from(&cc);
                        } else {
                            state.handle_char_class(cc)?;
                        }
                    } else if state.in_char_class {
                        // If in char class, push char to range buffer.
                        state.append_char_range_buf(c);
                    } else {
                        // If not in char class, handle as literal.
                        state.handle_literal_char(c)?;
                    }
                }
            }

            next = chars.next();
        }

        if expr.is_empty() {
            state.op_stack.push(Operator::EmptyPlaceholder);
        }

        while !state.op_stack.is_empty() {
            state.reduce_stack()?;
        }

        let head = state.stack.into_iter().last();
        Ok(head)
    }
}

/// Set of valid operators.
#[derive(Debug, PartialEq)]
pub enum Operator {
    Union,
    Concatenation,
    KleeneStar,
    Plus,
    Optional,
    LeftParen,
    EmptyPlaceholder,
}

#[derive(Debug)]
struct ParserState<T, SF, RF>
where
    SF: Copy + FnMut(&mut Vec<T>, &mut Vec<Operator>, CharClass) -> Result<()>,
    RF: Copy + FnMut(&mut Vec<T>, &mut Vec<Operator>) -> Result<()>,
{
    stack: Vec<T>,
    op_stack: Vec<Operator>,
    paren_count_stack: Vec<usize>,

    escaped: bool,
    insert_concat: bool,

    in_char_class: bool,
    char_class_buf: (CharClass, bool),
    char_range_buf: CharRangeBuf,

    shift_action: SF,
    reduce_action: RF,
}

#[derive(Debug)]
struct CharRangeBuf(Option<char>, Option<char>, Option<char>);

impl CharRangeBuf {
    #[inline]
    fn new() -> Self {
        CharRangeBuf(None, None, None)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.0 == None
    }

    #[inline]
    fn clear(&mut self) {
        self.0 = None;
        self.1 = None;
        self.2 = None;
    }
}

impl<T, SF, RF> ParserState<T, SF, RF>
where
    SF: Copy + FnMut(&mut Vec<T>, &mut Vec<Operator>, CharClass) -> Result<()>,
    RF: Copy + FnMut(&mut Vec<T>, &mut Vec<Operator>) -> Result<()>,
{
    #[inline]
    fn new(shift_action: SF, reduce_action: RF) -> Self {
        Self {
            stack: Vec::new(),
            op_stack: Vec::new(),
            paren_count_stack: Vec::new(),

            escaped: false,
            insert_concat: false,

            in_char_class: false,
            char_class_buf: (CharClass::new(), false),
            char_range_buf: CharRangeBuf::new(),

            shift_action,
            reduce_action,
        }
    }

    #[inline]
    fn handle_literal_char(&mut self, c: char) -> Result<()> {
        let char_class = c.into();
        self.handle_char_class(char_class)
    }

    #[inline]
    fn handle_char_class(&mut self, c: CharClass) -> Result<()> {
        while self.precedence_reduce_stack(&Operator::Concatenation)? {}

        if self.insert_concat {
            self.push_operator(Operator::Concatenation);
        }

        self.shift_action(c)?;
        self.insert_concat = true;

        Ok(())
    }

    #[inline]
    fn handle_union(&mut self) -> Result<()> {
        let op = Operator::Union;
        self.precedence_reduce_stack(&op)?;

        self.op_stack.push(op);
        self.insert_concat = false;

        Ok(())
    }

    #[inline]
    fn handle_kleene_star(&mut self) -> Result<()> {
        let op = Operator::KleeneStar;
        self.precedence_reduce_stack(&op)?;

        self.op_stack.push(op);
        self.insert_concat = true;

        Ok(())
    }

    #[inline]
    fn handle_plus(&mut self) -> Result<()> {
        let op = Operator::Plus;
        self.precedence_reduce_stack(&op)?;

        self.op_stack.push(op);
        self.insert_concat = true;

        Ok(())
    }

    #[inline]
    fn handle_optional(&mut self) -> Result<()> {
        let op = Operator::Optional;
        self.precedence_reduce_stack(&op)?;

        self.op_stack.push(op);
        self.insert_concat = true;

        Ok(())
    }

    #[inline]
    fn handle_left_paren(&mut self) -> Result<()> {
        let op = Operator::LeftParen;
        self.precedence_reduce_stack(&op)?;

        if self.insert_concat {
            self.push_concatenation();
        }

        self.op_stack.push(op);
        self.paren_count_stack.push(self.stack.len());
        self.insert_concat = false;

        Ok(())
    }

    #[inline]
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

    #[inline]
    fn handle_right_bracket(&mut self) -> Result<()> {
        // End char class if not escaped and in char class.
        self.in_char_class = false;

        // Throw error if nothing specified between brackets.
        if self.char_range_buf.is_empty() && self.char_class_buf.0.ranges.is_empty() {
            return Err(ParseError::EmptyCharacterClass);
        }

        self.handle_incomplete_char_range_buf();

        // Call shift action on completed char class.
        let char_class = if self.char_class_buf.1 {
            self.char_class_buf.0.complement()
        } else {
            self.char_class_buf.0.clone()
        };
        self.handle_char_class(char_class)?;

        // Clear the char class buffer.
        self.clear_char_class_buf();

        Ok(())
    }

    #[inline]
    fn handle_incomplete_char_range_buf(&mut self) {
        // Existing chars in first and second spots of buffer are added to
        // char class as single-char ranges.
        let s0 = self.char_range_buf.0;
        if let Some(s) = s0 {
            self.char_class_buf.0.add_range(CharRange::new_single(s));
            let s1 = self.char_range_buf.1;
            if let Some(s) = s1 {
                self.char_class_buf.0.add_range(CharRange::new_single(s));
            }
        }

        // Clear the char range buffer.
        self.char_range_buf.clear();
    }

    /// This method should only be called when in_char_class is true.
    /// The escaping of character class metasymbols (]) should be handled outside of this method
    /// call.
    #[inline]
    fn append_char_range_buf(&mut self, c: char) {
        if self.char_range_buf.0 == None {
            // If first spot is empty, add this char as the start of the range.
            self.char_range_buf.0 = Some(c);
        } else if self.char_range_buf.1 == None {
            if c == '-' {
                // If second spot is empty and this char is a dash, fill second spot.
                self.char_range_buf.1 = Some(c);
            } else {
                // If second spot is empty but this char is not a dash, add a single-char range to
                // the char class buffer.
                let new_range_char = self.char_range_buf.0.unwrap();
                let new_range = CharRange::new_single(new_range_char);
                self.char_class_buf.0.add_range(new_range);

                // Clear the range buffer.
                self.char_range_buf.clear();

                // Retry appending this char.
                self.append_char_range_buf(c);
            }
        } else if self.char_range_buf.2 == None {
            // If third spot is empty, complete the range and add it to the char class buffer.
            let start = self.char_range_buf.0.unwrap();
            let end = c;
            let new_range = CharRange::new(start, end);
            self.char_class_buf.0.add_range(new_range);

            self.char_range_buf.clear();
        }
        // There should never be a situation where all spots are filled.
    }

    #[inline]
    fn clear_char_class_buf(&mut self) {
        self.char_class_buf = (CharClass::new(), false);
    }

    #[inline]
    fn reduce_stack(&mut self) -> Result<()> {
        self.reduce_action()
    }

    #[inline]
    fn precedence_reduce_stack(&mut self, op: &Operator) -> Result<bool> {
        let reduce = match self.op_stack.last() {
            Some(last_op) => {
                if last_op == op && *last_op != Operator::LeftParen {
                    // If current op is the same as last, collapse the last.
                    // If both of left parenthesis, do nothing
                    true
                } else if *op == Operator::Union {
                    // If current op is alternation, collapse last if it is concat, kleene, plus,
                    // or optional.
                    *last_op == Operator::Concatenation
                        || *last_op == Operator::KleeneStar
                        || *last_op == Operator::Plus
                        || *last_op == Operator::Optional
                } else if *op == Operator::Concatenation {
                    // If current op is concat, collapse last if it is kleene, plus, or optional.
                    *last_op == Operator::KleeneStar
                        || *last_op == Operator::Plus
                        || *last_op == Operator::Optional
                } else if *op == Operator::KleeneStar
                    || *op == Operator::Plus
                    || *op == Operator::Optional
                {
                    // If current op is kleene star, plus, or optional, do not collapse last
                    // because they are highest precedence.
                    false
                } else if *op == Operator::LeftParen {
                    // If current op is left parenthesis, collapse last if it is kleene star, plus,
                    // or optional, which operate only on left node.
                    *last_op == Operator::KleeneStar
                        || *last_op == Operator::Plus
                        || *last_op == Operator::Optional
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

    #[inline]
    fn push_operator(&mut self, op: Operator) {
        self.op_stack.push(op);
    }

    #[inline]
    fn push_concatenation(&mut self) {
        self.op_stack.push(Operator::Concatenation);
    }

    #[inline]
    fn shift_action(&mut self, c: CharClass) -> Result<()> {
        (self.shift_action)(&mut self.stack, &mut self.op_stack, c)
    }

    #[inline]
    fn reduce_action(&mut self) -> Result<()> {
        (self.reduce_action)(&mut self.stack, &mut self.op_stack)
    }
}

/// Error returned when attempting to parse an invalid regular expression.
#[derive(Debug)]
pub enum ParseError {
    /// There are an invalid number of operators, or operands are missing.
    UnbalancedOperators,
    /// There are one or more sets of unclosed parentheses.
    UnbalancedParentheses,
    /// Bracketed character classes may not empty.
    EmptyCharacterClass,
}

impl fmt::Display for ParseError {
    #[inline]
    fn fmt<'a>(&self, f: &mut fmt::Formatter<'a>) -> fmt::Result {
        match *self {
            Self::UnbalancedOperators => write!(f, "unbalanced operators"),
            Self::UnbalancedParentheses => write!(f, "unbalanced parentheses"),
            Self::EmptyCharacterClass => write!(f, "empty character class"),
        }
    }
}

impl error::Error for ParseError {
    #[inline]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

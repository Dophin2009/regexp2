use crate::class::CharClass;

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Unary(UnaryOp, Box<Self>),
    Binary(BinaryOp, Box<Self>, Box<Self>),
    Atom(CharClass),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnaryOp {
    Star,
    Plus,
    Optional,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinaryOp {
    Concat,
    Alternate,
}

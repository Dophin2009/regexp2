pub type ASTNode<T> = Node<T, Operator>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Node<T, U> {
    Leaf(T),
    Branch(U, Box<Self>, Box<Self>),
    None,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Operator {
    KleeneStar,
    Plus,
    Optional,
    Concatenation,
    Union,
}

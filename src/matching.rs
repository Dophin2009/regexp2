use std::ops::Range;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Match<T> {
    /// Start position of the match.
    pub start: usize,
    /// Position of the last character matched + 1.
    pub end: usize,
    pub span: Vec<T>,
}

impl<T> Match<T> {
    #[inline]
    pub fn new(start: usize, end: usize, span: Vec<T>) -> Self {
        Match { start, end, span }
    }

    #[inline]
    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }
}

use std::ops::Range;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Match<T> {
    pub start: usize,
    pub end: usize,
    pub span: Vec<T>,
}

impl<T> Match<T> {
    pub fn new(start: usize, end: usize, span: Vec<T>) -> Self {
        Match { start, end, span }
    }

    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }
}

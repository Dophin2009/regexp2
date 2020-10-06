use std::ops::Range;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Match {
    start: usize,
    end: usize,
}

impl Match {
    pub fn new(start: usize, end: usize) -> Self {
        Match { start, end }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }
}

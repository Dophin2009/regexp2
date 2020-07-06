use crate::ranges::{DECIMAL_NUMBER, LETTER};
use std::cmp;
use std::convert::TryInto;
use std::hash::Hash;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CharClass {
    pub ranges: Vec<CharRange>,
}

impl CharClass {
    pub fn new() -> Self {
        Self::new_ranges(Vec::new())
    }

    pub fn new_range(range: CharRange) -> Self {
        Self::new_ranges(vec![range])
    }

    pub fn new_ranges(ranges: Vec<CharRange>) -> Self {
        Self { ranges }
    }

    pub fn new_single(c: char) -> Self {
        Self::new_range(CharRange::new_single(c))
    }

    pub fn contains(&self, c: char) -> bool {
        self.ranges.iter().any(|r| r.contains(c))
    }

    pub fn complement(&self) -> Self {
        let new_ranges = self
            .ranges
            .iter()
            .map(|r| {
                let complement = r.complement();
                // println!("{:?}", complement);
                complement
            })
            // Union of intersection of
            .fold_first(|union: Vec<CharRange>, complement| {
                println!("{:?}", union);
                union
                    .iter()
                    .flat_map(|ur: &CharRange| -> Vec<CharRange> {
                        complement
                            .iter()
                            .flat_map(|cr| ur.intersection(cr))
                            .collect()
                    })
                    .collect()
            });
        Self::new_ranges(new_ranges.unwrap_or(Vec::new()))
    }

    pub fn add_range(&mut self, range: CharRange) {
        self.ranges.push(range);
    }

    pub fn letter() -> Self {
        let ranges = LETTER.iter().map(|&r| r.into()).collect();
        Self { ranges }
    }

    pub fn decimal_number() -> Self {
        let ranges = DECIMAL_NUMBER.iter().map(|&r| r.into()).collect();
        Self { ranges }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CharRange {
    pub start: char,
    pub end: char,
}

impl CharRange {
    pub fn new(start: char, end: char) -> Self {
        CharRange { start, end }
    }
    pub fn new_single(c: char) -> Self {
        CharRange { start: c, end: c }
    }

    pub fn contains(&self, c: char) -> bool {
        self.start <= c && c <= self.end
    }

    pub fn intersection(&self, other: &Self) -> Option<Self> {
        if other.start > self.end || self.start > other.end {
            None
        } else {
            let start = cmp::max(self.start, other.start);
            let end = cmp::min(self.end, other.end);
            Some(Self::new(start, end))
        }
    }

    pub fn complement(&self) -> Vec<Self> {
        let mut ranges = Vec::new();

        let shift_char = |c, up: bool| {
            let shifted = if up { c as u32 + 1 } else { c as u32 - 1 };
            shifted.try_into().unwrap()
        };

        if self.start > '\u{e000}' {
            let r1 = Self::new('\u{e000}', shift_char(self.start, false));
            ranges.push(r1);

            let r2 = Self::new('\u{0}', '\u{d7ff}');
            ranges.push(r2);
        } else if self.start > '\u{0}' {
            let r = Self::new('\u{0}', shift_char(self.start, false));
            ranges.push(r);
        }

        if self.end < '\u{d7ff}' {
            let r1 = Self::new(shift_char(self.end, true), '\u{d7ff}');
            ranges.push(r1);

            let r2 = Self::new('\u{e000}', '\u{10ffff}');
            ranges.push(r2);
        } else if self.end < '\u{10ffff}' {
            let r = Self::new(shift_char(self.end, true), '\u{10ffff}');
            ranges.push(r);
        }

        ranges
    }
}

impl From<(char, char)> for CharRange {
    fn from(range: (char, char)) -> Self {
        CharRange {
            start: range.0,
            end: range.1,
        }
    }
}

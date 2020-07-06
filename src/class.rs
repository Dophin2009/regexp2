use crate::ranges::{DECIMAL_NUMBER, LETTER};
use std::cmp;
use std::convert::TryInto;
use std::hash::Hash;

const USV_START_1: char = '\u{0}';
const USV_END_1: char = '\u{d7ff}';
const USV_START_2: char = '\u{e000}';
const USV_END_2: char = '\u{10ffff}';

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

    pub fn copy_into(dest: &mut CharClass, src: &CharClass) {
        for r in src.ranges.clone() {
            dest.add_range(r);
        }
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

    pub fn all_but_newline() -> Self {
        let ranges = CharRange::new('\n', '\n').complement();
        Self::new_ranges(ranges)
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

        if self.start > USV_START_2 {
            let r1 = Self::new(USV_START_2, shift_char(self.start, false));
            ranges.push(r1);

            let r2 = Self::new(USV_START_1, USV_END_1);
            ranges.push(r2);
        } else if self.start > USV_START_1 {
            let r = Self::new(USV_START_1, shift_char(self.start, false));
            ranges.push(r);
        }

        if self.end < USV_END_1 {
            let r1 = Self::new(shift_char(self.end, true), USV_END_1);
            ranges.push(r1);

            let r2 = Self::new(USV_START_2, USV_END_2);
            ranges.push(r2);
        } else if self.end < USV_END_2 {
            let r = Self::new(shift_char(self.end, true), USV_END_2);
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

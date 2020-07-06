use crate::ranges::{DECIMAL_NUMBER, LETTER};
use std::cmp;
use std::convert::TryInto;
use std::hash::Hash;

/// The lowest Unicode scalar value.
const USV_START_1: char = '\u{0}';
/// The upper limit of the lower interval of Unicode scalar values.
const USV_END_1: char = '\u{d7ff}';
/// The lower limit of the upper interval of Unicode scalar values.
const USV_START_2: char = '\u{e000}';
/// The upper limit of the upper interval of Unicode scalar values.
const USV_END_2: char = '\u{10ffff}';

/// A set of character ranges that represent one character class. A CharClass contains all the
/// ranges in a single bracketed segment of character ranges in a regular expression.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CharClass {
    /// The ranges included in the character class.
    pub ranges: Vec<CharRange>,
}

impl CharClass {
    /// Create an empty character class.
    pub fn new() -> Self {
        Self::new_ranges(Vec::new())
    }

    /// Create a character class with a single range.
    pub fn new_range(range: CharRange) -> Self {
        Self::new_ranges(vec![range])
    }

    /// Create a character class with the given collection of ranges.
    pub fn new_ranges(ranges: Vec<CharRange>) -> Self {
        Self { ranges }
    }

    /// Create a character class with a one single-character character range.
    pub fn new_single(c: char) -> Self {
        Self::new_range(CharRange::new_single(c))
    }

    /// Create a character class with a set of single-character character ranges.
    pub fn new_singles(mut chars: Vec<char>) -> Self {
        chars.sort();
        chars.dedup();
        let ranges = chars.iter().map(|&c| CharRange::new_single(c)).collect();
        Self::new_ranges(ranges)
    }

    /// Add the ranges in the source CharClass to the destination CharClass.
    pub fn copy_into(dest: &mut CharClass, src: &CharClass) {
        for r in src.ranges.clone() {
            dest.add_range(r);
        }
    }

    /// Determine if the given char is within any of the character class's ranges.
    pub fn contains(&self, c: char) -> bool {
        self.ranges.iter().any(|r| r.contains(c))
    }

    /// Return the complement of the union of the ranges in the character class.
    pub fn complement(&self) -> Self {
        let new_ranges = self
            .ranges
            .iter()
            .map(|r| r.complement())
            // Union of intersection of
            .fold_first(|union: Vec<CharRange>, complement| {
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

    /// Add a character range to the set.
    pub fn add_range(&mut self, range: CharRange) {
        self.ranges.push(range);
    }

    /// Create a character class of all characters except the newline character.
    pub fn all_but_newline() -> Self {
        let ranges = CharRange::new('\n', '\n').complement();
        Self::new_ranges(ranges)
    }

    /// Create a character class consisting of all Unicode letter values.
    pub fn letter() -> Self {
        let ranges = LETTER.iter().map(|&r| r.into()).collect();
        Self::new_ranges(ranges)
    }

    /// Create a character class consisting of all alphanumerics and the underscore.
    pub fn word() -> Self {
        let ranges = vec![
            CharRange::new('A', 'Z'),
            CharRange::new('a', 'z'),
            CharRange::new('0', '9'),
            CharRange::new('_', '_'),
        ];
        Self::new_ranges(ranges)
    }

    /// Create a character class consisting of all Unicode decimal numbers.
    pub fn decimal_number() -> Self {
        let ranges = DECIMAL_NUMBER.iter().map(|&r| r.into()).collect();
        Self::new_ranges(ranges)
    }

    /// Create a character class consisting of whitespace characters.
    pub fn whitespace() -> Self {
        let mut cc = CharClass::new_singles(vec![
            ' ', '\u{000c}', '\n', '\r', '\t', '\u{000b}', '\u{00a0}', '\u{1680}', '\u{2028}',
            '\u{2029}', '\u{202f}', '\u{205f}', '\u{3000}', '\u{feff}',
        ]);
        let tmp_range = CharClass::new_range(CharRange::new('\u{2000}', '\u{200a}'));
        CharClass::copy_into(&mut cc, &tmp_range);
        cc
    }
}

/// A range of characters representing all characters from the lower bound to the upper bound,
/// inclusive.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CharRange {
    pub start: char,
    pub end: char,
}

impl CharRange {
    /// Create a new character range with the given bounds.
    pub fn new(start: char, end: char) -> Self {
        CharRange { start, end }
    }

    /// Create a single-character character range for the given character.
    pub fn new_single(c: char) -> Self {
        CharRange { start: c, end: c }
    }

    /// Determine if the given character is within the range.
    pub fn contains(&self, c: char) -> bool {
        self.start <= c && c <= self.end
    }

    /// Return the range that is the intersection between two ranges.
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        if other.start > self.end || self.start > other.end {
            None
        } else {
            let start = cmp::max(self.start, other.start);
            let end = cmp::min(self.end, other.end);
            Some(Self::new(start, end))
        }
    }

    /// Return the set of ranges that equals the complement of this range. Because Unicode scalar
    /// values, which `char` encodes, consist of all Unicode code points except high-surrogate and
    /// low-surrogate code points, characters between the values of 0xD7FF and 0xE000, exclusive,
    /// are omitted.
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
    /// Create a character range from a tuple, where the first element is the lower bound, and the
    /// second element is the upper bound.
    fn from(range: (char, char)) -> Self {
        CharRange {
            start: range.0,
            end: range.1,
        }
    }
}

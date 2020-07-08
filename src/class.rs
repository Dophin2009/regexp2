use crate::disjoint::{self, DisjointSet, Intersect, Priority};
use crate::ranges::{DECIMAL_NUMBER, LETTER};

use std::cmp;
use std::convert::TryInto;
use std::hash::Hash;
use std::iter;

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
    pub ranges: DisjointSet<char, CharRange>,
}

impl CharClass {
    /// Determine if the given char is within any of the character class's ranges.
    pub fn contains(&self, c: char) -> bool {
        self.ranges.iter().any(|r| r.contains(c))
    }

    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    // Union of the intersections of each range in `Self` with each range in `other`.
    pub fn intersection(&self, other: &Self) -> Self {
        self.iter().fold(CharClass::new(), |mut union, self_r| {
            let intersections = other
                .iter()
                .flat_map(|other_r: &CharRange| self_r.intersection(other_r));
            union.extend(intersections);
            union
        })
    }

    /// Return the complement of the union of the ranges in the character class.
    pub fn complement(&self) -> Self {
        self.iter()
            .map(|r| r.complement().into())
            .fold_first(|union: CharClass, complement| union.intersection(&complement))
            .unwrap_or(CharClass::new())
    }

    /// Copy the ranges in `other` to this `Self`.
    pub fn copy_from(&mut self, other: &CharClass) {
        for r in other {
            self.add_range(r.clone());
        }
    }

    /// Add a character range to the set.
    pub fn add_range(&mut self, range: CharRange) {
        self.ranges.insert(range);
    }
}

impl CharClass {
    /// Create a character class of all characters except the newline character.
    pub fn all_but_newline() -> Self {
        CharRange::new('\n', '\n').complement().into()
    }

    /// Create a character class consisting of all Unicode letter values.
    pub fn letter() -> Self {
        LETTER.iter().map(|&r| r.into()).collect()
    }

    /// Create a character class consisting of all alphanumerics and the underscore.
    pub fn word() -> Self {
        let ranges = vec![
            CharRange::new('A', 'Z'),
            CharRange::new('a', 'z'),
            CharRange::new('0', '9'),
            CharRange::new('_', '_'),
        ];
        ranges.into()
    }

    /// Create a character class consisting of all Unicode decimal numbers.
    pub fn decimal_number() -> Self {
        DECIMAL_NUMBER.iter().map(|&r| r.into()).collect()
    }

    /// Create a character class consisting of whitespace characters.
    pub fn whitespace() -> Self {
        let chars = vec![
            ' ', '\u{000c}', '\n', '\r', '\t', '\u{000b}', '\u{00a0}', '\u{1680}', '\u{2028}',
            '\u{2029}', '\u{202f}', '\u{205f}', '\u{3000}', '\u{feff}',
        ];

        let mut cc: Self = chars.into();
        cc.add_range(CharRange::new('\u{2000}', '\u{200a}'));
        cc
    }
}

impl CharClass {
    /// Create an empty character class.
    pub fn new() -> Self {
        Self {
            ranges: DisjointSet::new(),
        }
    }
}

impl From<CharRange> for CharClass {
    /// Create a character class with a single range.
    fn from(range: CharRange) -> Self {
        let mut class = CharClass::new();
        class.add_range(range);
        class
    }
}

impl From<char> for CharClass {
    /// Create a character class with one single-character character range.
    fn from(c: char) -> Self {
        CharRange::from(c).into()
    }
}

impl From<Vec<CharRange>> for CharClass {
    fn from(vec: Vec<CharRange>) -> Self {
        let mut class = CharClass::new();
        class.extend(vec);
        class
    }
}

impl From<Vec<char>> for CharClass {
    fn from(vec: Vec<char>) -> Self {
        let mut class = CharClass::new();
        class.extend(vec.into_iter().map(|c| CharRange::from(c)));
        class
    }
}

impl Extend<CharRange> for CharClass {
    fn extend<I: IntoIterator<Item = CharRange>>(&mut self, iter: I) {
        for r in iter {
            self.add_range(r);
        }
    }
}

impl iter::FromIterator<CharRange> for CharClass {
    fn from_iter<I: IntoIterator<Item = CharRange>>(iter: I) -> Self {
        let mut class = Self::new();
        class.extend(iter);
        class
    }
}

impl CharClass {
    pub fn iter(&self) -> CharClassIter {
        self.ranges.iter().into()
    }

    pub fn iter_mut(&mut self) -> CharClassIterMut {
        self.ranges.iter_mut().into()
    }
}

impl<'a> IntoIterator for &'a CharClass {
    type Item = &'a CharRange;
    type IntoIter = CharClassIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.ranges.iter().into()
    }
}

impl<'a> IntoIterator for &'a mut CharClass {
    type Item = &'a mut CharRange;
    type IntoIter = CharClassIterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.ranges.iter_mut().into()
    }
}

impl IntoIterator for CharClass {
    type Item = CharRange;
    type IntoIter = CharClassIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.ranges.into_iter().into()
    }
}

pub struct CharClassIter<'a> {
    set_iter: disjoint::Iter<'a, char, CharRange>,
}

impl<'a> Iterator for CharClassIter<'a> {
    type Item = &'a CharRange;

    fn next(&mut self) -> Option<Self::Item> {
        self.set_iter.next()
    }
}

impl<'a> From<disjoint::Iter<'a, char, CharRange>> for CharClassIter<'a> {
    fn from(set_iter: disjoint::Iter<'a, char, CharRange>) -> Self {
        Self { set_iter }
    }
}

pub struct CharClassIterMut<'a> {
    set_iter: disjoint::IterMut<'a, char, CharRange>,
}

impl<'a> Iterator for CharClassIterMut<'a> {
    type Item = &'a mut CharRange;

    fn next(&mut self) -> Option<Self::Item> {
        self.set_iter.next()
    }
}

impl<'a> From<disjoint::IterMut<'a, char, CharRange>> for CharClassIterMut<'a> {
    fn from(set_iter: disjoint::IterMut<'a, char, CharRange>) -> Self {
        Self { set_iter }
    }
}

pub struct CharClassIntoIter {
    set_iter: disjoint::IntoIter<char, CharRange>,
}

impl Iterator for CharClassIntoIter {
    type Item = CharRange;

    fn next(&mut self) -> Option<Self::Item> {
        self.set_iter.next()
    }
}

impl From<disjoint::IntoIter<char, CharRange>> for CharClassIntoIter {
    fn from(set_iter: disjoint::IntoIter<char, CharRange>) -> Self {
        Self { set_iter }
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

impl Intersect for CharRange {
    fn intersect(&self, other: &Self) -> bool {
        self.intersection(other).is_some()
    }

    fn union(&self, other: &Self) -> Self {
        Self::new(
            cmp::min(self.start, other.start),
            cmp::max(self.end, other.end),
        )
    }
}

impl Priority<char> for CharRange {
    fn priority(&self) -> char {
        self.start
    }
}

impl From<char> for CharRange {
    fn from(c: char) -> Self {
        Self::new(c, c)
    }
}

impl From<(char, char)> for CharRange {
    /// Create a character range from a tuple, where the first element is the lower bound, and the
    /// second element is the upper bound.
    fn from(range: (char, char)) -> Self {
        Self::new(range.0, range.1)
    }
}

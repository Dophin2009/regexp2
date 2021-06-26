use crate::matching::Match;
use crate::table::Table;

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter::Peekable;
use std::rc::Rc;

/// A deterministic finite automaton, or DFA.
#[derive(Debug, Clone)]
pub struct DFA<T>
where
    T: Clone + Eq + Hash,
{
    /// A DFA has a single start state.
    pub start_state: usize,
    /// The number of total states in the DFA. There is a state labeled i for every i where 0 <= i
    /// < total_states.
    pub total_states: usize,
    /// The set of accepting states.
    pub accepting_states: HashSet<usize>,
    /// A lookup table for transitions between states.
    pub transition: Table<usize, Transition<T>, usize>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Transition<T>(pub T)
where
    T: Clone + Eq + Hash;

impl<T> From<T> for Transition<T>
where
    T: Clone + Eq + Hash,
{
    #[inline]
    fn from(t: T) -> Self {
        Self(t)
    }
}

impl<T> DFA<T>
where
    T: Clone + Eq + Hash,
{
    /// Create a new DFA with a single start state.
    #[inline]
    pub fn new() -> Self {
        Self {
            start_state: 0,
            total_states: 1,
            accepting_states: HashSet::new(),
            transition: Table::new(),
        }
    }
}

impl<T> Default for DFA<T>
where
    T: Clone + Eq + Hash,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> DFA<T>
where
    T: Clone + Eq + Hash,
{
    #[inline]
    pub fn add_state(&mut self, is_final: bool) -> usize {
        let label = self.total_states;
        self.total_states += 1;
        if is_final {
            self.accepting_states.insert(label);
        }
        label
    }

    #[inline]
    pub fn add_transition<U>(&mut self, start: usize, end: usize, label: U) -> Option<()>
    where
        U: Into<Transition<T>>,
    {
        if self.total_states < start + 1 || self.total_states < end + 1 {
            None
        } else {
            self.transition.set(start, label.into(), end);
            Some(())
        }
    }

    #[inline]
    pub fn transitions_on(&self, state: &usize) -> HashMap<&Transition<T>, &usize> {
        self.transition.get_row(state)
    }

    #[inline]
    pub fn is_accepting_state(&self, state: &usize) -> bool {
        self.accepting_states.iter().any(|s| s == state)
    }
}

impl<T> DFA<T>
where
    T: Clone + Eq + Hash,
{
    #[inline]
    pub fn iter_on<I>(&self, input: I) -> Iter<'_, T, I::IntoIter>
    where
        T: PartialEq<I::Item>,
        I: IntoIterator,
    {
        Iter {
            dfa: &self,

            input: input.into_iter().peekable(),
            last: None,
        }
    }

    #[inline]
    pub fn into_iter_on<I>(self, input: I) -> IntoIter<T, I::IntoIter>
    where
        T: PartialEq<I::Item>,
        I: IntoIterator,
    {
        IntoIter {
            dfa: self,

            input: input.into_iter().peekable(),
            last: None,
        }
    }
}

pub struct Iter<'a, T, I>
where
    T: Clone + Eq + Hash,
    T: PartialEq<I::Item>,
    I: Iterator,
{
    dfa: &'a DFA<T>,

    input: Peekable<I>,
    last: Option<(LastIterState, usize)>,
}

impl<'a, T, I> Iterator for Iter<'a, T, I>
where
    T: Clone + Eq + Hash,
    T: PartialEq<I::Item>,
    I: Iterator,
{
    type Item = IterState<I>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        iter_on_next(&self.dfa, &mut self.input, &mut self.last)
    }
}

pub struct IntoIter<T, I>
where
    T: Clone + Eq + Hash,
    T: PartialEq<I::Item>,
    I: Iterator,
{
    dfa: DFA<T>,

    input: Peekable<I>,
    last: Option<(LastIterState, usize)>,
}

impl<T, I> Iterator for IntoIter<T, I>
where
    T: Clone + Eq + Hash,
    T: PartialEq<I::Item>,
    I: Iterator,
{
    type Item = IterState<I>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        iter_on_next(&self.dfa, &mut self.input, &mut self.last)
    }
}

#[derive(Debug)]
pub enum IterState<I>
where
    I: Iterator,
{
    Start(usize, bool),
    Normal(I::Item, usize, bool),
    Stuck(usize),
}

enum LastIterState {
    Start,
    Normal,
    Stuck,
}

#[inline]
fn iter_on_next<T, I>(
    dfa: &DFA<T>,
    input: &mut Peekable<I>,
    last: &mut Option<(LastIterState, usize)>,
) -> Option<IterState<I>>
where
    T: Clone + Eq + Hash,
    T: PartialEq<I::Item>,
    I: Iterator,
{
    let current = match *last {
        // First iteration, return start.
        None => {
            *last = Some((LastIterState::Start, dfa.start_state));
            return Some(IterState::Start(
                dfa.start_state,
                dfa.is_accepting_state(&dfa.start_state),
            ));
        }
        Some((LastIterState::Start | LastIterState::Normal, state)) => state,
        // If we were last stuck, return None to indicate that last state was stuck.
        Some((LastIterState::Stuck, _)) => return None,
    };

    let peek_is = match input.peek() {
        Some(v) => v,
        // No more input, so last item was the final.
        None => return None,
    };

    let transitions = dfa.transitions_on(&current);
    let next = match transitions
        .iter()
        .find(|(&Transition(t), _)| *t == *peek_is)
    {
        Some((_, &&next_state)) => {
            // Consume input symbol.
            let is = input.next().unwrap();

            // Check if current state is an accepting one.
            let is_final = dfa.is_accepting_state(&next_state);

            *last = Some((LastIterState::Normal, next_state));
            Some(IterState::Normal(is, next_state, is_final))
        }
        // No more transitions, so stuck.
        None => {
            *last = Some((LastIterState::Stuck, current));
            Some(IterState::Stuck(current))
        }
    };

    next
}

impl<T> DFA<T>
where
    T: Clone + Eq + Hash,
{
    /// Determine if the given input is accepted by the DFA.
    #[inline]
    pub fn is_match<I>(&self, input: I) -> bool
    where
        T: PartialEq<I::Item>,
        I: IntoIterator,
    {
        match self.iter_on(input).last() {
            Some(IterState::Start(_, is_final) | IterState::Normal(_, _, is_final)) => is_final,
            Some(IterState::Stuck(_)) => false,
            // None means that no movement happened; check if the current state (start state) is an
            // accepting state.
            None => unreachable!(),
        }
    }

    #[inline]
    pub fn find_shortest<I>(&self, input: I) -> Option<Match<I::Item>>
    where
        T: PartialEq<I::Item>,
        I: IntoIterator,
    {
        self.find_shortest_at(input, 0)
    }

    #[inline]
    pub fn find_shortest_at<I>(&self, input: I, start: usize) -> Option<Match<I::Item>>
    where
        T: PartialEq<I::Item>,
        I: IntoIterator,
    {
        self.find_at_impl(input, start, true)
    }

    #[inline]
    pub fn find<I>(&self, input: I) -> Option<Match<I::Item>>
    where
        T: PartialEq<I::Item>,
        I: IntoIterator,
    {
        self.find_at(input, 0)
    }

    #[inline]
    pub fn find_at<I>(&self, input: I, start: usize) -> Option<Match<I::Item>>
    where
        T: PartialEq<I::Item>,
        I: IntoIterator,
    {
        self.find_at_impl(input, start, false)
    }

    #[inline]
    fn find_at_impl<I>(&self, input: I, start: usize, shortest: bool) -> Option<Match<I::Item>>
    where
        T: PartialEq<I::Item>,
        I: IntoIterator,
    {
        let mut last_match = None;
        let iter = self.iter_on(input).skip(start).enumerate();

        // Ensure span dropped before unwrapping Rc's.
        {
            let mut span = Vec::new();
            for (i, iter_state) in iter {
                let is_final = match iter_state {
                    IterState::Start(_, is_final) => is_final,
                    IterState::Normal(is, _, is_final) => {
                        let is_rc = Rc::new(is);
                        span.push(is_rc);

                        is_final
                    }
                    IterState::Stuck(_) => break,
                };

                if is_final {
                    last_match = Some(Match::new(start, i + 1, span.clone()));
                    if shortest {
                        break;
                    }
                }
            }
        }

        last_match.map(|m| {
            Match::new(
                m.start(),
                m.end(),
                m.span
                    .into_iter()
                    .map(|rc| match Rc::try_unwrap(rc) {
                        Ok(v) => v,
                        // Shouldn't ever have any lingering references.
                        Err(_) => unreachable!("MatchRc somehow had lingering references"),
                    })
                    .collect(),
            )
        })
    }
}

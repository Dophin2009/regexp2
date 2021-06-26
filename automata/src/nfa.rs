use crate::matching::Match;
use crate::table::Table;

use std::borrow::Cow;
use std::hash::Hash;
use std::iter::Peekable;
use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

include!("macros.rs");

/// A non-deterministic finite automaton, or NFA.
#[derive(Clone, Debug)]
pub struct NFA<T: Clone + Eq + Hash> {
    /// An NFA has a single start state.
    pub start_state: usize,
    /// The number of total states in the NFA. There is a state labeled i for every i where 0 <= i
    /// < total_states.
    pub total_states: usize,
    /// The set of accepting states.
    pub accepting_states: HashSet<usize>,
    /// A lookup table for transitions between states.
    pub transition: Table<usize, Transition<T>, HashSet<usize>>,
}

/// A transition between states in an NFA.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Transition<T: Clone + Eq + Hash> {
    /// A transition on some input symbol.
    Some(T),
    /// An epsilon transition allows the NFA to change its state spontaneously without consuming an
    /// input symbol.
    Epsilon,
}

impl<T> NFA<T>
where
    T: Clone + Eq + Hash,
{
    /// Create a new NFA with a single start state.
    #[allow(clippy::new_without_default)]
    #[inline]
    pub fn new() -> Self {
        NFA {
            start_state: 0,
            total_states: 1,
            accepting_states: HashSet::new(),
            transition: Table::new(),
        }
    }

    /// Create a new NFA with an start state, a single accepting state, and an epsilon transition
    /// between them.
    #[inline]
    pub fn new_epsilon() -> Self {
        let mut nfa = NFA::new();
        let accepting_state = nfa.add_state(true);
        nfa.add_epsilon_transition(nfa.start_state, accepting_state);

        nfa
    }

    /// Clone the states and transitions of an NFA into another. The start and accepting states of the
    /// source are not marked as such in the destination. These states can be accessed by i +
    /// offset, where i is the label of the state in the source NFA, and offset is the start
    /// total number of states in the destination NFA.
    #[inline]
    pub fn copy_into(dest: &mut NFA<T>, src: &NFA<T>) {
        let offset = dest.total_states;
        // Create new states.
        for _ in 0..src.total_states {
            dest.add_state(false);
        }

        // Clone the transitions.
        for (start, label, ends) in src.transition.into_iter() {
            for end in ends {
                dest.add_transition(*start + offset, *end + offset, (*label).clone());
            }
        }
    }

    /// Construct a new NFA for the union operator of two NFAs. There are epsilon transitions
    /// from the start state and initial states of the operands. There are also epsilon
    /// transitions from each accepting state of the operands to the final state.
    #[inline]
    pub fn union(c1: &NFA<T>, c2: &NFA<T>) -> NFA<T> {
        let mut new_nfa = NFA::new();
        let accepting_state = new_nfa.add_state(true);
        let start_state = new_nfa.start_state;

        let mut offset = new_nfa.total_states;

        NFA::copy_into(&mut new_nfa, c1);
        new_nfa.add_epsilon_transition(start_state, c1.start_state + offset);
        for c1_final in c1.accepting_states.iter() {
            new_nfa.add_epsilon_transition(*c1_final + offset, accepting_state);
        }

        offset = new_nfa.total_states;

        NFA::copy_into(&mut new_nfa, c2);
        new_nfa.add_epsilon_transition(start_state, c2.start_state + offset);
        for c2_final in c2.accepting_states.iter() {
            new_nfa.add_epsilon_transition(*c2_final + offset, accepting_state);
        }

        new_nfa
    }

    /// Construct a new NFA for the concatenation operator of two NFAs. The start state of the
    /// preceding NFA becomes the start state of the new NFA. The accepting states of the following NFA
    /// are the accepting states of the new NFA. There are epsilon transitions from the final states of
    /// the former to the start state of the latter.
    #[inline]
    pub fn concatenation(c1: &NFA<T>, c2: &NFA<T>) -> NFA<T> {
        let mut new_nfa = c1.clone();

        let offset = new_nfa.total_states;
        NFA::copy_into(&mut new_nfa, &c2);

        // Epsilon transitions from c1 finals to start of c2
        for c1_final in c1.accepting_states.iter() {
            new_nfa.add_epsilon_transition(*c1_final, c2.start_state + offset);
        }
        new_nfa.accepting_states = HashSet::new();

        // Set accepting states
        for c2_final in c2.accepting_states.iter() {
            new_nfa.accepting_states.insert(c2_final + offset);
        }

        new_nfa
    }

    /// Construct a new NFA for the kleene star operator of an NFA.
    #[inline]
    pub fn kleene_star(c1: &NFA<T>) -> NFA<T> {
        let mut new_nfa = NFA::new_epsilon();
        let offset = new_nfa.total_states;

        NFA::copy_into(&mut new_nfa, &c1);
        new_nfa.add_epsilon_transition(new_nfa.start_state, c1.start_state + offset);

        for c1_final in c1.accepting_states.iter() {
            new_nfa.add_epsilon_transition(c1_final + offset, c1.start_state + offset);
            for accepting_state in new_nfa.accepting_states.clone().iter() {
                new_nfa.add_epsilon_transition(c1_final + offset, *accepting_state);
            }
        }

        new_nfa
    }

    /// Construct a new NFA with epsilon transitions from the start state to the initial states
    /// of each child. The accepting states of the new NFA are the final states of the children.
    #[inline]
    pub fn combine(cc: &[&NFA<T>]) -> NFA<T> {
        let mut new_nfa = NFA::new();
        let mut offset = new_nfa.total_states;
        for c in cc {
            NFA::copy_into(&mut new_nfa, c);
            new_nfa.add_epsilon_transition(new_nfa.start_state, c.start_state + offset);

            for c_final in c.accepting_states.iter() {
                new_nfa.accepting_states.insert(c_final + offset);
            }
            offset += c.total_states;
        }

        new_nfa
    }

    /// Add a state to the NFA. The label of the state is returned. The total number of states is
    /// always greater than the label of the newest state by 1.
    #[inline]
    pub fn add_state(&mut self, is_final: bool) -> usize {
        let label = self.total_states;
        if is_final {
            self.accepting_states.insert(label);
        }

        self.total_states += 1;
        label
    }

    /// Add a transition. Returns None if one or more of the states does not exist.
    #[inline]
    pub fn add_transition(&mut self, start: usize, end: usize, label: Transition<T>) -> Option<()> {
        if self.total_states < start + 1 || self.total_states < end + 1 {
            None
        } else {
            self.transition.set_or(start, label, hash_set![end], |v| {
                v.insert(end);
            });
            Some(())
        }
    }

    // Add a non-epsilon transition. See [add_transition].
    #[inline]
    pub fn add_labeled_transition(&mut self, start: usize, end: usize, label: T) -> Option<()> {
        self.add_transition(start, end, Transition::Some(label))
    }

    // Add an epsilon transition. See [add_transition].
    #[inline]
    pub fn add_epsilon_transition(&mut self, start: usize, end: usize) -> Option<()> {
        self.add_transition(start, end, Transition::Epsilon)
    }

    #[inline]
    pub fn is_accepting_state(&self, label: &usize) -> bool {
        self.accepting_states.contains(label)
    }

    /// Returns the transitions and destinations from a specific state.
    #[inline]
    pub fn transitions_from(&self, state: usize) -> HashMap<&Transition<T>, &HashSet<usize>> {
        self.transition.get_row(&state)
    }

    /// Computes the function epsilon-closure for some given state in the NFA. Returns the set of
    /// all states accessible from the given state on epsilon transitions only.
    #[inline]
    pub fn epsilon_closure(&self, state: usize) -> HashSet<usize> {
        let transitions = self.transitions_from(state);
        let mut closure: HashSet<_> = transitions
            .into_iter()
            .filter(|(t, _)| **t == Transition::Epsilon)
            .flat_map(|(_, dest)| dest.iter().flat_map(|&i| self.epsilon_closure(i)))
            .collect();
        closure.insert(state);
        closure
    }

    /// Computes the union of epsilon-closures for each state in the given set of states.
    #[inline]
    pub fn epsilon_closure_set(&self, state_set: &HashSet<usize>) -> HashSet<usize> {
        let mut set = state_set.clone();
        for state in state_set.iter() {
            let state_closure = self.epsilon_closure(*state);
            set = set.union(&state_closure).cloned().collect();
        }
        set
    }

    #[inline]
    fn move_set<S>(&self, state_set: &HashSet<usize>, input: &S) -> HashSet<usize>
    where
        T: PartialEq<S>,
    {
        let mut set = HashSet::new();
        for state in state_set.iter() {
            let transitions = self.transitions_from(*state);
            let input_transitions = transitions
                .into_iter()
                .filter(|(t, _)| match *t {
                    Transition::Some(symbol) => *symbol == *input,
                    Transition::Epsilon => false,
                })
                .flat_map(|(_, dest)| dest.iter().cloned())
                .collect();
            set = set.union(&input_transitions).cloned().collect();
        }
        set
    }

    #[inline]
    pub fn iter_on<I>(&self, input: I) -> Iter<'_, T, I::IntoIter>
    where
        I: IntoIterator,
        T: PartialEq<I::Item>,
    {
        Iter {
            nfa: &self,
            input: input.into_iter().peekable(),
            last: None,
        }
    }

    #[inline]
    pub fn into_iter_on<I>(self, input: I) -> IntoIter<T, I::IntoIter>
    where
        I: IntoIterator,
        T: PartialEq<I::Item>,
    {
        IntoIter {
            nfa: self,
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
    nfa: &'a NFA<T>,

    input: Peekable<I>,
    last: Option<(bool, HashSet<usize>)>,
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
        iter_on_next(&self.nfa, &mut self.input, &mut self.last)
    }
}

pub struct IntoIter<T, I>
where
    T: Clone + Eq + Hash,
    T: PartialEq<I::Item>,
    I: Iterator,
{
    nfa: NFA<T>,

    input: Peekable<I>,
    last: Option<(bool, HashSet<usize>)>,
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
        iter_on_next(&self.nfa, &mut self.input, &mut self.last)
    }
}

#[derive(Debug)]
pub enum IterState<I>
where
    I: Iterator,
{
    Normal(I::Item, HashMap<usize, bool>),
    Stuck(HashSet<usize>),
}

#[inline]
fn iter_on_next<T, I>(
    nfa: &NFA<T>,
    input: &mut Peekable<I>,
    last: &mut Option<(bool, HashSet<usize>)>,
) -> Option<IterState<I>>
where
    T: Clone + Eq + Hash,
    T: PartialEq<I::Item>,
    I: Iterator,
{
    let current_set = match last {
        None => Cow::Owned(nfa.epsilon_closure(nfa.start_state)),
        Some((false, set)) => Cow::Borrowed(set),
        // If we were last stuck, return None to indicate that last state was stuck.
        Some((true, _)) => return None,
    };

    let peek_is = match input.peek() {
        Some(v) => v,
        // No more input, so last item was the final.
        None => return None,
    };

    let moved_set = nfa.move_set(&current_set, peek_is);
    let next_set = nfa.epsilon_closure_set(&moved_set);

    let next = if !next_set.is_empty() {
        // Consume input symbol.
        let is = input.next().unwrap();

        // Check if states are accepting ones.
        let next_set_map = next_set
            .iter()
            .map(|s| (*s, nfa.is_accepting_state(s)))
            .collect();

        *last = Some((false, next_set));
        Some(IterState::Normal(is, next_set_map))
    } else {
        *last = Some((true, next_set.clone()));
        Some(IterState::Stuck(next_set))
    };

    next
}

impl<T> NFA<T>
where
    T: Clone + Eq + Hash,
{
    /// Determines if the given input is accepted by the NFA.
    #[inline]
    pub fn is_match<I>(&self, input: I) -> bool
    where
        T: PartialEq<I::Item>,
        I: IntoIterator,
    {
        let mut state_set = self.epsilon_closure(self.start_state);

        for is in input.into_iter() {
            let moved_set = self.move_set(&state_set, &is);
            state_set = self.epsilon_closure_set(&moved_set);
        }

        state_set.iter().any(|s| self.is_accepting_state(s))
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
        let mut last_match = if self.is_accepting_state(&self.start_state) {
            Some(Match::new(start, start, vec![]))
        } else {
            None
        };

        if !(shortest && last_match.is_some()) {
            let mut state_set = self.epsilon_closure(self.start_state);

            let input = input.into_iter().skip(start);
            let mut span = Vec::new();
            for (i, is) in input.enumerate() {
                let moved_set = self.move_set(&state_set, &is);
                state_set = self.epsilon_closure_set(&moved_set);

                let is_rc = Rc::new(is);
                span.push(is_rc);

                if state_set.iter().any(|s| self.is_accepting_state(s)) {
                    last_match = Some(Match::new(start, i + 1, span.clone()));
                    if shortest {
                        break;
                    }
                }
            }
        }

        last_match.map(|m| {
            Match::new(
                m.start,
                m.end,
                m.span
                    .into_iter()
                    .map(|rc| match Rc::try_unwrap(rc) {
                        Ok(v) => v,
                        // Shouldn't ever have any lingering references.
                        Err(_) => unreachable!("Match Rc somehow had lingering references"),
                    })
                    .collect(),
            )
        })
    }
}

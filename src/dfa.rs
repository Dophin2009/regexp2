use crate::matching::Match;
use crate::nfa::{self, NFA};
use crate::table::Table;

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::iter::Peekable;
use std::rc::Rc;

/// Must be implemented by NFA transition symbol types to ensure each DFA state has only one
/// possible transition on any symbol.
pub trait Disjoin: Sized {
    /// Given a set of transition symbols, return a set of non-overlapping transition symbols.
    fn disjoin(vec: Vec<&Self>) -> Vec<Self>;

    fn contains(&self, other: &Self) -> bool;
}

/// A deterministic finite automaton, or DFA.
#[derive(Debug, Clone)]
pub struct DFA<T>
where
    T: Clone + Eq + Hash,
{
    /// A DFA has a single initial state.
    pub initial_state: usize,
    /// The number of total states in the DFA. There is a state labeled i for every i where 0 <= i
    /// < total_states.
    pub total_states: usize,
    /// The set of accepting states.
    pub final_states: HashSet<usize>,
    /// A lookup table for transitions between states.
    pub transition: Table<usize, Transition<T>, usize>,
}

#[derive(Debug)]
pub struct DFAFromNFA<T>
where
    T: Clone + Eq + Hash,
{
    pub dfa: DFA<T>,
    pub nfa_mapping: HashMap<usize, HashSet<usize>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Transition<T>(pub T)
where
    T: Clone + Eq + Hash;

impl<T> DFA<T>
where
    T: Clone + Eq + Hash,
{
    /// Create a new DFA with a single initial state.
    pub fn new() -> Self {
        Self {
            initial_state: 0,
            total_states: 1,
            final_states: HashSet::new(),
            transition: Table::new(),
        }
    }
}

impl<T> Default for DFA<T>
where
    T: Clone + Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> DFA<T>
where
    T: Clone + Eq + Hash,
{
    pub fn add_state(&mut self, is_final: bool) -> usize {
        let label = self.total_states;
        self.total_states += 1;
        if is_final {
            self.final_states.insert(label);
        }
        label
    }

    pub fn add_transition(&mut self, start: usize, end: usize, label: Transition<T>) -> Option<()> {
        if self.total_states < start + 1 || self.total_states < end + 1 {
            None
        } else {
            self.transition.set(start, label, end);
            Some(())
        }
    }

    pub fn is_final_state(&self, state: &usize) -> bool {
        self.final_states.iter().any(|s| s == state)
    }
}

struct MatchRc<T> {
    start: usize,
    end: usize,
    span: Vec<Rc<T>>,
}

impl<T> MatchRc<T> {
    fn new(start: usize, end: usize, span: Vec<Rc<T>>) -> Self {
        Self { start, end, span }
    }
}

impl<T> DFA<T>
where
    T: Clone + Eq + Hash,
{
    /// Determine if the given input is accepted by the DFA.
    pub fn is_match<I>(&self, input: I) -> bool
    where
        T: PartialEq<I::Item>,
        I: IntoIterator,
    {
        let mut state = self.initial_state;

        for is in input.into_iter() {
            let transitions = self.transition.get_row(&state);
            state = match transitions.iter().find(|(&Transition(t), _)| *t == is) {
                Some((_, &&s)) => s,
                // No transition on current symbol from current state: no match.
                None => return false,
            }
        }

        self.is_final_state(&state)
    }

    pub fn has_match<I>(&self, input: I) -> bool
    where
        T: PartialEq<I::Item>,
        I: IntoIterator,
    {
        self.has_match_at(input, 0)
    }

    pub fn has_match_at<I>(&self, input: I, start: usize) -> bool
    where
        T: PartialEq<I::Item>,
        I: IntoIterator,
    {
        self.find_shortest_at(input, start).is_some()
    }

    pub fn find_shortest<I>(&self, input: I) -> Option<(Match<I::Item>, usize)>
    where
        T: PartialEq<I::Item>,
        I: IntoIterator,
    {
        self.find_shortest_at(input, 0)
    }

    pub fn find_shortest_at<I>(&self, input: I, start: usize) -> Option<(Match<I::Item>, usize)>
    where
        T: PartialEq<I::Item>,
        I: IntoIterator,
    {
        self._find_at(input, start, true)
    }

    pub fn find<I>(&self, input: I) -> Option<(Match<I::Item>, usize)>
    where
        T: PartialEq<I::Item>,
        I: IntoIterator,
    {
        self.find_at(input, 0)
    }

    pub fn find_at<I>(&self, input: I, start: usize) -> Option<(Match<I::Item>, usize)>
    where
        T: PartialEq<I::Item>,
        I: IntoIterator,
    {
        self._find_at(input, start, false)
    }

    fn _find_at<I>(&self, input: I, start: usize, shortest: bool) -> Option<(Match<I::Item>, usize)>
    where
        T: PartialEq<I::Item>,
        I: IntoIterator,
    {
        let mut state = self.initial_state;
        let mut last_match = if self.is_final_state(&state) {
            Some(MatchRc::new(start, start, vec![]))
        } else {
            None
        };

        if !(shortest && last_match.is_some()) {
            let input = input.into_iter().skip(start);
            let mut span = Vec::new();
            for (i, is) in input.enumerate() {
                let transitions = self.transition.get_row(&state);
                state = match transitions.iter().find(|(&Transition(t), _)| *t == is) {
                    Some((_, &&s)) => s,
                    // No transition on current symbol from current state: no match.
                    None => break,
                };

                let is_rc = Rc::new(is);
                span.push(is_rc);

                if self.is_final_state(&state) {
                    last_match = Some(MatchRc::new(start, i + 1, span.clone()));
                    if shortest {
                        break;
                    }
                }
            }
        }

        last_match.map(|m| {
            let mt = Match::new(
                m.start,
                m.end,
                m.span
                    .into_iter()
                    .map(|rc| match Rc::try_unwrap(rc) {
                        Ok(v) => v,
                        // Shouldn't ever have any lingering references.
                        Err(_) => unreachable!(),
                    })
                    .collect(),
            );
            (mt, state)
        })
    }

    pub fn find_shortest_mut<I>(&self, input: &mut Peekable<I>) -> Option<(Match<I::Item>, usize)>
    where
        T: PartialEq<I::Item>,
        I: Iterator,
    {
        self._find_mut(input, true)
    }

    pub fn find_mut<I>(&self, input: &mut Peekable<I>) -> Option<(Match<I::Item>, usize)>
    where
        T: PartialEq<I::Item>,
        I: Iterator,
    {
        self._find_mut(input, false)
    }

    fn _find_mut<I>(
        &self,
        input: &mut Peekable<I>,
        shortest: bool,
    ) -> Option<(Match<I::Item>, usize)>
    where
        T: PartialEq<I::Item>,
        I: Iterator,
    {
        let mut state = self.initial_state;
        let mut last_match = if self.is_final_state(&state) {
            Some(MatchRc::new(0, 0, vec![]))
        } else {
            None
        };

        if !(shortest && last_match.is_some()) {
            let mut span = Vec::new();

            let mut i = 0;
            // Peek the next symbol to check if a transition on it exists.
            // If there's no transition, break and do not consume that symbol.
            // If there is a transition, consume the symbol and push it to the span.
            while let Some(is_next) = input.peek() {
                // Find the transition (if it exists) from the current state for the next symbol.
                let transitions = self.transition.get_row(&state);
                state = match transitions
                    .iter()
                    .find(|(&Transition(t), _)| *t == *is_next)
                {
                    // Transition found, change the current state to the new state.
                    Some((_, &&s)) => s,
                    // No transition on next symbol from current state: no further match to be
                    // found.
                    None => break,
                };

                // Actually consume the next symbol from the iterator and push it to the span.
                let is = input.next().unwrap();
                i += 1;

                let is_rc = Rc::new(is);
                span.push(is_rc);

                if self.is_final_state(&state) {
                    last_match = Some(MatchRc::new(0, i + 1, span.clone()));
                    if shortest {
                        break;
                    }
                }
            }
        }

        last_match.map(|m| {
            let mt = Match::new(
                m.start,
                m.end,
                m.span
                    .into_iter()
                    .map(|rc| match Rc::try_unwrap(rc) {
                        Ok(v) => v,
                        // Shouldn't ever have any lingering references.
                        Err(_) => unreachable!(),
                    })
                    .collect(),
            );
            (mt, state)
        })
    }
}

#[derive(Clone, Debug)]
struct DState {
    label: usize,
    nfa_states: HashSet<usize>,
}

impl DState {
    fn new(label: usize, nfa_states: HashSet<usize>) -> Self {
        DState { label, nfa_states }
    }
}

impl<T> From<NFA<T>> for DFA<T>
where
    T: Clone + Disjoin + Eq + Hash,
{
    fn from(nfa: NFA<T>) -> Self {
        let dfa_from_nfa: DFAFromNFA<T> = nfa.into();
        dfa_from_nfa.into()
    }
}

impl<T> From<DFAFromNFA<T>> for DFA<T>
where
    T: Clone + Disjoin + Eq + Hash,
{
    fn from(dfa_from_nfa: DFAFromNFA<T>) -> Self {
        dfa_from_nfa.dfa
    }
}

impl<T> From<NFA<T>> for DFAFromNFA<T>
where
    T: Clone + Disjoin + Eq + Hash,
{
    // Create an equivalent DFA from an NFA using the subset construction described by Algorithm
    // 3.20. The construction is slightly modified, with inspiration from [this Stack Overflow
    //   answer](https://stackoverflow.com/a/25832898/8955108) to accomodate character ranges.
    fn from(nfa: NFA<T>) -> Self {
        let mut dfa = DFA::new();
        let mut nfa_mapping = HashMap::new();

        let mut marked_states = Vec::new();
        let mut unmarked_states = VecDeque::new();

        let label = 0;
        let initial_e_closure = nfa.epsilon_closure(nfa.initial_state);
        let initial_unmarked = DState::new(label, initial_e_closure);

        if initial_unmarked
            .nfa_states
            .iter()
            .any(|i| nfa.is_final_state(i))
        {
            dfa.final_states.insert(initial_unmarked.label);
        }

        nfa_mapping.insert(initial_unmarked.label, initial_unmarked.nfa_states.clone());
        unmarked_states.push_back(initial_unmarked);

        while let Some(s) = unmarked_states.pop_front() {
            // Get all non-epsilon transitions and destinations from the NFA states in this set
            // state.
            let transition_map: Vec<(&T, &HashSet<usize>)> = s
                .clone()
                .nfa_states
                .into_iter()
                // Union of transitions from each NFA state
                .flat_map(|nfa_state| nfa.transitions_from(nfa_state))
                // Filter out epsilon transitions
                .filter_map(|(t, v)| match t {
                    nfa::Transition::Some(a) => Some((a, v)),
                    nfa::Transition::Epsilon => None,
                })
                .collect();

            // Isolate transitions.
            let transitions: Vec<&T> = transition_map.iter().map(|(t, _)| *t).collect();
            // Disjoin transitions.
            let disjoint_transitions = T::disjoin(transitions);

            for t in disjoint_transitions {
                let moved_set: HashSet<usize> = transition_map
                    .iter()
                    .filter(|(a, _)| a.contains(&t))
                    .flat_map(|(_, v)| (*v).clone())
                    .collect();
                let epsilon_closure = nfa.epsilon_closure_set(&moved_set);
                let mut new_state = DState::new(0, epsilon_closure);

                // If state already exists in unmarked or marked, change the label and do not push
                // to unmarked.
                if s.nfa_states == new_state.nfa_states {
                    new_state.label = s.label;
                    dfa.add_transition(s.label, new_state.label, Transition(t));
                } else if let Some(existing) = marked_states
                    .iter()
                    .find(|ss: &&DState| ss.nfa_states == new_state.nfa_states)
                {
                    new_state.label = existing.label;
                    dfa.add_transition(s.label, new_state.label, Transition(t));
                } else if let Some(existing) = unmarked_states
                    .iter()
                    .find(|ss: &&DState| ss.nfa_states == new_state.nfa_states)
                {
                    new_state.label = existing.label;
                    dfa.add_transition(s.label, new_state.label, Transition(t));
                } else {
                    // If not found, set a new label and push to unmarked.
                    new_state.label = dfa.add_state(false);

                    // If this set state contains an accepting NFA state, set this set state
                    // as accepting in the DFA.
                    if new_state.nfa_states.iter().any(|i| nfa.is_final_state(i)) {
                        dfa.final_states.insert(new_state.label);
                    }

                    dfa.add_transition(s.label, new_state.label, Transition(t));
                    nfa_mapping.insert(new_state.label, new_state.nfa_states.clone());
                    unmarked_states.push_back(new_state);
                }
            }

            // Mark this state
            marked_states.push(s);
        }

        DFAFromNFA { dfa, nfa_mapping }
    }
}

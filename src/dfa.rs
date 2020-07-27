use crate::matching::Match;
use crate::nfa::{self, NFA};
use crate::table::Table;

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

/// Must be implemented by NFA transition symbol types to ensure each DFA state has only one
/// possible transition on any symbol.
pub trait Disjoin: Sized {
    /// Given a set of transition symbols, return a set of non-overlapping transition symbols.
    fn disjoin(vec: Vec<&Self>) -> Vec<Self>;

    fn contains(&self, other: &Self) -> bool;
}

/// A deterministic finite automaton, or DFA.
#[derive(Debug)]
pub struct DFA<T>
where
    T: Clone + Eq + Hash,
{
    /// A DFA has a single initial state.
    pub initial_state: u32,
    /// The number of total states in the DFA. There is a state labeled i for every i where 0 <= i
    /// < total_states.
    pub total_states: u32,
    /// The set of accepting states.
    pub final_states: HashSet<u32>,
    /// A lookup table for transitions between states.
    pub transition: Table<u32, Transition<T>, u32>,
}

#[derive(Debug)]
pub struct DFAFromNFA<T>
where
    T: Clone + Eq + Hash,
{
    pub dfa: DFA<T>,
    pub nfa_mapping: HashMap<u32, HashSet<u32>>,
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

    pub fn add_state(&mut self, is_final: bool) -> u32 {
        let label = self.total_states;
        self.total_states += 1;
        if is_final {
            self.final_states.insert(label);
        }
        label
    }

    pub fn add_transition(&mut self, start: u32, end: u32, label: Transition<T>) -> Option<()> {
        if self.total_states < start + 1 || self.total_states < end + 1 {
            None
        } else {
            self.transition.set(start, label, end);
            Some(())
        }
    }

    pub fn is_final_state(&self, state: &u32) -> bool {
        self.final_states.iter().any(|s| s == state)
    }
}

impl<T> DFA<T>
where
    T: Clone + Eq + Hash,
{
    /// Determine if the given input is accepted by the DFA.
    pub fn is_match<I>(&self, input: &I) -> bool
    where
        T: PartialEq<I::Item>,
        I: Clone + IntoIterator,
    {
        let mut state = self.initial_state;

        for is in input.clone().into_iter() {
            let transitions = self.transition.get_row(&state);
            state = match transitions.iter().find(|(&Transition(t), _)| *t == is) {
                Some((_, &&s)) => s,
                // No transition on current symbol from current state: no match.
                None => return false,
            }
        }

        self.is_final_state(&state)
    }

    pub fn has_match<I>(&self, input: &I) -> bool
    where
        T: PartialEq<I::Item>,
        I: Clone + IntoIterator,
    {
        self.has_match_at(input, 0)
    }

    pub fn has_match_at<I>(&self, input: &I, start: usize) -> bool
    where
        T: PartialEq<I::Item>,
        I: Clone + IntoIterator,
    {
        self.find_shortest_at(input, start).is_some()
    }

    pub fn find_shortest<I>(&self, input: &I) -> Option<(Match, u32)>
    where
        T: PartialEq<I::Item>,
        I: Clone + IntoIterator,
    {
        self.find_shortest_at(input, 0)
    }

    pub fn find_shortest_at<I>(&self, input: &I, start: usize) -> Option<(Match, u32)>
    where
        T: PartialEq<I::Item>,
        I: Clone + IntoIterator,
    {
        self._find_at(input, start, true)
    }

    pub fn find<I>(&self, input: &I) -> Option<(Match, u32)>
    where
        T: PartialEq<I::Item>,
        I: Clone + IntoIterator,
    {
        self.find_at(input, 0)
    }

    pub fn find_at<I>(&self, input: &I, start: usize) -> Option<(Match, u32)>
    where
        T: PartialEq<I::Item>,
        I: Clone + IntoIterator,
    {
        self._find_at(input, start, false)
    }

    fn _find_at<I>(&self, input: &I, start: usize, shortest: bool) -> Option<(Match, u32)>
    where
        T: PartialEq<I::Item>,
        I: Clone + IntoIterator,
    {
        let mut state = self.initial_state;
        let mut last_match = if self.is_final_state(&state) {
            Some(Match::new(start, start))
        } else {
            None
        };

        if shortest && last_match.is_some() {
            return last_match.map(|m| (m, state));
        }

        let input = input.clone().into_iter().skip(start);
        for (i, is) in input.enumerate() {
            let transitions = self.transition.get_row(&state);
            state = match transitions.iter().find(|(&Transition(t), _)| *t == is) {
                Some((_, &&s)) => s,
                // No transition on current symbol from current state: no match.
                None => break,
            };

            if self.is_final_state(&state) {
                last_match = Some(Match::new(start, i));
                if shortest {
                    break;
                }
            }
        }

        last_match.and_then(|m| Some((m, state)))
    }
}

#[derive(Clone, Debug)]
struct DState {
    label: u32,
    nfa_states: HashSet<u32>,
}

impl DState {
    fn new(label: u32, nfa_states: HashSet<u32>) -> Self {
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
            let transition_map: Vec<(&T, &HashSet<u32>)> = s
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
                let moved_set: HashSet<u32> = transition_map
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

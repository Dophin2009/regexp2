use crate::nfa::{self, NFA};
use crate::table::Table;
use std::collections::{HashSet, VecDeque};
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
    pub inital_state: u32,
    /// The number of total states in the DFA. There is a state labeled i for every i where 0 <= i
    /// < total_states.
    pub total_states: u32,
    /// The set of accepting states.
    pub final_states: HashSet<u32>,
    /// A lookup table for transitions between states.
    pub transition: Table<u32, Transition<T>, u32>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Transition<T>(T)
where
    T: Clone + Eq + Hash;

impl<T> DFA<T>
where
    T: Clone + Eq + Hash,
{
    /// Create a new DFA with a single initial state.
    pub fn new() -> Self {
        Self {
            inital_state: 0,
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

    /// Determine if the given input is accepted by the DFA.
    pub fn is_exact_match<S, I>(&self, input: I) -> bool
    where
        T: PartialEq<S>,
        I: Iterator<Item = S>,
    {
        let iter = self.iter_input(input);
        let final_state = iter.last();
        match final_state {
            Some(op_s) => match op_s {
                Some(s) => self.final_states.contains(&s),
                None => false,
            },
            None => false,
        }
    }

    /// Produce a DFAIterator for the DFA on some input iterator. See [DFAIterator].
    pub fn iter_input<'a, S, I>(&'a self, input: I) -> DFAIterator<'a, T, S, I>
    where
        T: PartialEq<S>,
        I: Iterator<Item = S>,
    {
        DFAIterator::new(self, input)
    }
}

/// An iterator on DFA states over some input. The values of the input iterator must be able to be
/// matched to transitions.
#[derive(Debug)]
pub struct DFAIterator<'a, T, S, I>
where
    T: Clone + Eq + Hash + PartialEq<S>,
    I: Iterator<Item = S>,
{
    input: I,
    current_state: Option<u32>,
    dfa: &'a DFA<T>,

    iter_c: u32,
}

impl<'a, T, S, I> Iterator for DFAIterator<'a, T, S, I>
where
    T: Clone + Eq + Hash + PartialEq<S>,
    I: Iterator<Item = S>,
{
    type Item = Option<u32>;

    /// Consume one input sybmol and advance the state of the DFA according to that symbol.
    fn next(&mut self) -> Option<Self::Item> {
        self.iter_c += 1;

        // On first iter, return initial state
        if self.iter_c == 1 {
            return Some(self.current_state);
        }

        match self.current_state {
            Some(current_state) => {
                let c = match self.input.next() {
                    Some(c) => c,
                    None => return None,
                };

                let transitions = self.dfa.transition.get_row(&current_state);
                self.current_state = match transitions.iter().find(|(&Transition(t), _v)| *t == c) {
                    Some((_, &&s)) => Some(s),
                    None => {
                        self.current_state = None;
                        return Some(None);
                    }
                };

                Some(self.current_state)
            }
            None => None,
        }
    }
}

impl<'a, T, S, I> DFAIterator<'a, T, S, I>
where
    T: Clone + Eq + Hash + PartialEq<S>,
    I: Iterator<Item = S>,
{
    /// Create a new DFAIterator on the given input for the given DFA.
    fn new(dfa: &'a DFA<T>, input: I) -> Self {
        DFAIterator {
            input,
            current_state: Some(dfa.inital_state),
            dfa,

            iter_c: 0,
        }
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
    T: Clone + Disjoin + Eq + Hash + std::fmt::Debug,
{
    // Create an equivalent DFA from an NFA.
    fn from(nfa: NFA<T>) -> Self {
        let mut dfa = DFA::new();
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

        unmarked_states.push_back(initial_unmarked);

        while let Some(s) = unmarked_states.pop_front() {
            println!("{:?}\n", s);

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
            println!("{:?}\n", disjoint_transitions);

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
                    unmarked_states.push_back(new_state);
                }
            }

            // Mark this state
            marked_states.push(s);
        }

        println!("{:#?}", dfa);

        dfa
    }
}

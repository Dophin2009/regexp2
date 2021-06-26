use crate::dfa::{Transition, DFA};
use crate::nfa::{self, NFA};

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::Hash;

/// Must be implemented by NFA transition symbol types to ensure each DFA state has only one
/// possible transition on any symbol.
pub trait Disjoin: Sized {
    /// Given a set of transition symbols, return a set of non-overlapping transition symbols.
    fn disjoin(vec: Vec<&Self>) -> Vec<Self>;

    fn contains(&self, other: &Self) -> bool;
}

#[derive(Debug)]
pub struct DFAFromNFA<T>
where
    T: Clone + Eq + Hash,
{
    pub dfa: DFA<T>,
    pub nfa_mapping: HashMap<usize, HashSet<usize>>,
}

#[derive(Clone, Debug)]
struct DState {
    label: usize,
    nfa_states: HashSet<usize>,
}

impl DState {
    #[inline]
    fn new(label: usize, nfa_states: HashSet<usize>) -> Self {
        Self { label, nfa_states }
    }
}

impl<T> From<NFA<T>> for DFA<T>
where
    T: Clone + Disjoin + Eq + Hash,
{
    #[inline]
    fn from(nfa: NFA<T>) -> Self {
        let dfa_from_nfa: DFAFromNFA<T> = nfa.into();
        dfa_from_nfa.into()
    }
}

impl<T> From<DFAFromNFA<T>> for DFA<T>
where
    T: Clone + Disjoin + Eq + Hash,
{
    #[inline]
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
    #[inline]
    fn from(nfa: NFA<T>) -> Self {
        let mut dfa = DFA::new();
        let mut nfa_mapping = HashMap::new();

        let mut marked_states = Vec::new();
        let mut unmarked_states = VecDeque::new();

        let label = 0;
        let initial_e_closure = nfa.epsilon_closure(nfa.start_state);
        let initial_unmarked = DState::new(label, initial_e_closure);

        if initial_unmarked
            .nfa_states
            .iter()
            .any(|i| nfa.is_accepting_state(i))
        {
            dfa.accepting_states.insert(initial_unmarked.label);
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
                    if new_state
                        .nfa_states
                        .iter()
                        .any(|i| nfa.is_accepting_state(i))
                    {
                        dfa.accepting_states.insert(new_state.label);
                    }

                    dfa.add_transition(s.label, new_state.label, Transition(t));
                    nfa_mapping.insert(new_state.label, new_state.nfa_states.clone());
                    unmarked_states.push_back(new_state);
                }
            }

            // Mark this state
            marked_states.push(s);
        }

        Self { dfa, nfa_mapping }
    }
}

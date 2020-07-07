use crate::nfa::{self, NFA};
use crate::table::Table;
use std::collections::{HashSet, VecDeque};
use std::hash::Hash;

/// Must be implemented by NFA transition symbol types to ensure each DFA state has only one
/// possible transition on any symbol.
pub trait Disjoin: Sized {
    /// Given a set of transition symbols, return a set of non-overlapping transition symbols.
    fn disjoin(vec: Vec<&Self>) -> Vec<Self>;
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
    // Create an equivalent DFA from an NFA.
    fn from(nfa: NFA<T>) -> Self {
        let mut dfa = DFA::new();
        let mut marked_states = Vec::new();
        let mut unmarked_states = VecDeque::new();

        let mut label = 0;

        let initial_e_closure = nfa.epsilon_closure(nfa.initial_state);
        let initial_unmarked = DState::new(label, initial_e_closure);
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
                    .filter(|(a, _)| **a == t)
                    .flat_map(|(_, v)| (*v).clone())
                    .collect();
                let epsilon_closure = nfa.epsilon_closure_set(&moved_set);
                let mut new_state = DState::new(0, epsilon_closure);

                // If state already exists in unmarked or marked, change the label and do not push
                // to unmarked.
                let mut push_unmarked = false;
                if s.nfa_states != new_state.nfa_states {
                    new_state.label = s.label;
                } else if let Some(existing) = marked_states
                    .iter()
                    .find(|ss: &&DState| ss.nfa_states == new_state.nfa_states)
                {
                    new_state.label = existing.label;
                } else if let Some(existing) = unmarked_states
                    .iter()
                    .find(|ss: &&DState| ss.nfa_states == new_state.nfa_states)
                {
                    new_state.label = existing.label;
                } else {
                    // If not found, set a new label and push to unmarked.
                    label += 1;
                    new_state.label = label;

                    // If this set state contains an accepting NFA state, set this set state
                    // as accepting in the DFA.
                    if new_state
                        .nfa_states
                        .iter()
                        .any(|i| nfa.final_states.contains(i))
                    {
                        dfa.final_states.insert(new_state.label);
                    }

                    push_unmarked = true;
                }

                dfa.transition.set(s.label, Transition(t), new_state.label);

                if push_unmarked {
                    unmarked_states.push_back(new_state);
                }
            }

            // Mark this state
            marked_states.push(s);
        }

        dfa
    }
}

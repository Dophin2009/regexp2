use crate::table::Table;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

// Use the regex-syntax crate to convert ranges of Unicode scalar values to equivalent sets of
// ranges of Unicode codepoints.
// use regex_syntax::utf8::{Utf8Sequence, Utf8Sequences};
#[derive(Debug)]
pub struct NFA<T: Clone + Eq + Hash> {
    initial_state: u32,
    total_states: u32,
    final_states: HashSet<u32>,
    transition: Table<u32, Transition<T>, Vec<u32>>,
}

// macro_rules! hash_set {
// () => {
// HashSet::new()
// };
// ( $( $x:expr ),* ) => {{
// let mut set = HashSet::new();
// $(set.insert($x);)*
// set
// }};
// }

impl<T: Clone + Eq + Hash> NFA<T> {
    /// Create a new NFA with a single initial state.
    pub fn new() -> Self {
        NFA {
            initial_state: 0,
            total_states: 1,
            final_states: HashSet::new(),
            transition: Table::new(),
        }
    }

    /// Create a new NFA with an initial state, a single final state, and an epsilon transition
    /// between them.
    pub fn new_epsilon() -> Self {
        let mut nfa = NFA::new();
        let final_state = nfa.add_state(true);
        nfa.add_epsilon_transition(nfa.initial_state, final_state);

        nfa
    }

    /// Clone the states and transitions of an NFA into another. The initial and final states of the
    /// source are not marked as such in the destination.
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
    /// from the initial state and initial states of the operands. There are also epsilon
    /// transitions from each final state of the operands to the final state.
    pub fn union(c1: &NFA<T>, c2: &NFA<T>) -> NFA<T> {
        let mut new_nfa = NFA::new();
        let final_state = new_nfa.add_state(true);
        let initial_state = new_nfa.initial_state;

        let mut offset = new_nfa.total_states;

        NFA::copy_into(&mut new_nfa, c1);
        new_nfa.add_epsilon_transition(initial_state, c1.initial_state + offset);
        for c1_final in c1.final_states.iter() {
            new_nfa.add_epsilon_transition(*c1_final + offset, final_state);
        }

        offset = new_nfa.total_states;

        NFA::copy_into(&mut new_nfa, c2);
        new_nfa.add_epsilon_transition(initial_state, c2.initial_state + offset);
        for c2_final in c2.final_states.iter() {
            new_nfa.add_epsilon_transition(*c2_final + offset, final_state);
        }

        new_nfa
    }

    /// Construct a new NFA for the concatenation operator of two NFAs. The start state of the
    /// preceding NFA becomes the start state of the new NFA. The final states of the following NFA
    /// are the final states of the new NFA. There are epsilon transitions from the final states of
    /// the former to the start state of the latter.
    pub fn concatenation(c1: &NFA<T>, c2: &NFA<T>) -> NFA<T> {
        let mut new_nfa = c1.clone();

        let offset = new_nfa.total_states;
        NFA::copy_into(&mut new_nfa, &c2);

        // Epsilon transitions from c1 finals to initial of c2
        for c1_final in c1.final_states.iter() {
            new_nfa.add_epsilon_transition(*c1_final, c2.initial_state + offset);
        }
        new_nfa.final_states = HashSet::new();

        // Set final states
        for c2_final in c2.final_states.iter() {
            new_nfa.final_states.insert(c2_final + offset);
        }

        new_nfa
    }

    /// Add a state to the NFA. The label of the state is returned. The total number of states is
    /// always greater than the label of the newest state by 1.
    pub fn add_state(&mut self, is_final: bool) -> u32 {
        let label = self.total_states;
        if is_final {
            self.final_states.insert(label);
        }

        self.total_states += 1;
        label
    }

    /// Add a transition. Returns None if one or more of the states does not exist.
    pub fn add_transition(&mut self, start: u32, end: u32, label: Transition<T>) -> Option<()> {
        if self.total_states < start + 1 || self.total_states < end + 1 {
            None
        } else {
            self.transition
                .set_or(start, label, vec![end], |v| v.push(end));
            Some(())
        }
    }

    pub fn add_labeled_transition(&mut self, start: u32, end: u32, label: T) -> Option<()> {
        self.add_transition(start, end, Transition::Some(label))
    }

    pub fn add_epsilon_transition(&mut self, start: u32, end: u32) -> Option<()> {
        self.add_transition(start, end, Transition::Epsilon)
    }
}

impl<T: Clone + Eq + Hash> Clone for NFA<T> {
    fn clone(&self) -> Self {
        NFA {
            total_states: self.total_states,
            initial_state: self.initial_state,
            final_states: self.final_states.clone(),
            transition: self.transition.clone(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Transition<T: Clone + Eq + Hash> {
    Some(T),
    Epsilon,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let n: NFA<bool> = NFA::new();

        assert_eq!(1, n.total_states);
        assert_eq!(0, n.initial_state);
        assert_eq!(0, n.final_states.len());
        assert_eq!(0, n.transition.into_iter().count());
    }

    #[test]
    fn test_new_epsilon() {
        let n: NFA<bool> = NFA::new_epsilon();

        assert_eq!(2, n.total_states);
        assert_eq!(0, n.initial_state);
        assert_eq!(1, n.final_states.len());

        assert_eq!(1, n.transition.into_iter().count());

        let (_, tran, _) = n.transition.into_iter().next().unwrap();
        assert_eq!(Transition::Epsilon, *tran);
    }

    #[test]
    fn test_add_state() {
        let mut n: NFA<bool> = NFA::new();
        let new_state = n.add_state(false);
        assert_eq!(2, n.total_states);
        assert_eq!(n.total_states - 1, new_state);
        assert_eq!(0, n.final_states.len());

        let mut n: NFA<bool> = NFA::new();
        let new_state = n.add_state(true);
        assert_eq!(2, n.total_states);
        assert_eq!(n.total_states - 1, new_state);
        assert_eq!(1, n.final_states.len());
    }

    #[test]
    fn test_union() {
        let c1: NFA<bool> = NFA::new_epsilon();
        let c2: NFA<bool> = NFA::new_epsilon();

        let union = NFA::union(&c1, &c2);
        assert_eq!(6, union.total_states);
        assert_eq!(1, union.final_states.len());
    }

    #[test]
    fn test_concatenation() {
        let c1: NFA<bool> = NFA::new_epsilon();
        let c2: NFA<bool> = NFA::new_epsilon();

        let concat = NFA::concatenation(&c1, &c2);
        assert_eq!(4, concat.total_states);
        assert_eq!(c2.final_states.len(), concat.final_states.len());
        assert_eq!(c1.initial_state, concat.initial_state);
    }
}

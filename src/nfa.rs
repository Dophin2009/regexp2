use crate::hash_set;
use crate::table::Table;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// A non-deterministic finite automaton, or NFA.
#[derive(Debug)]
pub struct NFA<T: Clone + Eq + Hash> {
    /// An NFA has a single initial state.
    pub initial_state: u32,
    /// The number of total states in the NFA. There is a state labeled i for every i where 0 <= i
    /// < total_states.
    pub total_states: u32,
    /// The set of accepting states.
    pub final_states: HashSet<u32>,
    /// A lookup table for transitions between states.
    pub transition: Table<u32, Transition<T>, HashSet<u32>>,
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
    /// source are not marked as such in the destination. These states can be accessed by i +
    /// offset, where i is the label of the state in the source NFA, and offset is the initial
    /// total number of states in the destination NFA.
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

    /// Construct a new NFA for the kleene star operator of an NFA.
    pub fn kleene_star(c1: &NFA<T>) -> NFA<T> {
        let mut new_nfa = NFA::new_epsilon();
        let offset = new_nfa.total_states;

        NFA::copy_into(&mut new_nfa, &c1);
        new_nfa.add_epsilon_transition(new_nfa.initial_state, c1.initial_state + offset);

        for c1_final in c1.final_states.iter() {
            new_nfa.add_epsilon_transition(c1_final + offset, c1.initial_state + offset);
            for final_state in new_nfa.final_states.clone().iter() {
                new_nfa.add_epsilon_transition(c1_final + offset, *final_state);
            }
        }

        new_nfa
    }

    /// Construct a new NFA with epsilon transitions from the initial state to the initial states
    /// of each child. The final states of the new NFA are the final states of the children.
    pub fn combine(cc: &Vec<&NFA<T>>) -> NFA<T> {
        let mut new_nfa = NFA::new();
        let mut offset = new_nfa.total_states;
        for c in cc {
            NFA::copy_into(&mut new_nfa, c);
            new_nfa.add_epsilon_transition(new_nfa.initial_state, c.initial_state + offset);

            for c_final in c.final_states.iter() {
                new_nfa.final_states.insert(c_final + offset);
            }
            offset += c.total_states;
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
            self.transition.set_or(start, label, hash_set![end], |v| {
                v.insert(end);
            });
            Some(())
        }
    }

    // Add a non-epsilon transition. See [add_transition].
    pub fn add_labeled_transition(&mut self, start: u32, end: u32, label: T) -> Option<()> {
        self.add_transition(start, end, Transition::Some(label))
    }

    // Add an epsilon transition. See [add_transition].
    pub fn add_epsilon_transition(&mut self, start: u32, end: u32) -> Option<()> {
        self.add_transition(start, end, Transition::Epsilon)
    }

    /// Computes the function epsilon-closure for some given state in the NFA. Returns the set of
    /// all states accessible from the given state on epsilon transitions only.
    pub fn epsilon_closure(&self, state: u32) -> HashSet<u32> {
        let transitions = self.transitions_from(state);
        let mut closure: HashSet<_> = transitions
            .into_iter()
            .filter(|(t, _)| **t == Transition::Epsilon)
            .flat_map(|(_, dest)| dest.into_iter().flat_map(|&i| self.epsilon_closure(i)))
            .collect();
        closure.insert(state);
        closure
    }

    /// Computes the union of epsilon-closures for each state in the given set of states.
    pub fn epsilon_closure_set(&self, state_set: &HashSet<u32>) -> HashSet<u32> {
        let mut set = state_set.clone();
        for state in state_set.iter() {
            let state_closure = self.epsilon_closure(*state);
            set = set.union(&state_closure).map(|&i| i).collect();
        }
        set
    }

    /// Returns the transitions and destinations from a specific state.
    pub fn transitions_from(&self, state: u32) -> HashMap<&Transition<T>, &HashSet<u32>> {
        self.transition.get_row(&state)
    }
}

impl<T: Clone + Eq + Hash> Clone for NFA<T> {
    /// Clone the NFA.
    fn clone(&self) -> Self {
        NFA {
            total_states: self.total_states,
            initial_state: self.initial_state,
            final_states: self.final_states.clone(),
            transition: self.transition.clone(),
        }
    }
}

impl<T> NFA<T>
where
    T: Clone + Eq + Hash,
{
    /// Produces an NFAIterator for the NFA on some input iterator. See [NFAIterator].
    pub fn iter_input<'a, S, I>(&'a self, input: I) -> NFAIterator<'a, T, S, I>
    where
        T: PartialEq<S>,
        I: Iterator<Item = S>,
    {
        NFAIterator::new(self, input)
    }

    /// Determines if the given input is accepted by the NFA.
    pub fn is_exact_match<'a, S, I>(&self, input: I) -> bool
    where
        T: PartialEq<S>,
        I: Iterator<Item = S>,
    {
        let iter = self.iter_input(input);
        let final_set = iter.last();
        match final_set {
            Some(set) => set.iter().any(|s| self.final_states.contains(s)),
            None => false,
        }
    }
}

/// An iterator on NFA states over some input. The values of the input
/// iterator must be able to be matched to transitions.
#[derive(Debug)]
pub struct NFAIterator<'a, T, S, I>
where
    T: Clone + Eq + Hash + PartialEq<S>,
    I: Iterator<Item = S>,
{
    input: I,
    state_set: HashSet<u32>,
    nfa: &'a NFA<T>,

    iter_c: u32,
}

impl<'a, T, S, I> Iterator for NFAIterator<'a, T, S, I>
where
    T: Clone + Eq + Hash + PartialEq<S>,
    I: Iterator<Item = S>,
{
    type Item = HashSet<u32>;

    /// Consumes one input symbol and advances the state of the NFA according to that symbol.
    fn next(&mut self) -> Option<Self::Item> {
        self.iter_c += 1;
        if self.iter_c == 1 {
            return Some(self.state_set.clone());
        }

        let c = match self.input.next() {
            Some(c) => c,
            None => return None,
        };

        let moved_set = self.move_set(&self.state_set, &c);
        self.state_set = self.nfa.epsilon_closure_set(&moved_set);

        Some(self.state_set.clone())
    }
}

impl<'a, T, S, I> NFAIterator<'a, T, S, I>
where
    T: Clone + Eq + Hash + PartialEq<S>,
    I: Iterator<Item = S>,
{
    /// Create a new NFAIterator on the given input for the given NFA.
    fn new(nfa: &'a NFA<T>, input: I) -> Self {
        NFAIterator {
            input,
            state_set: nfa.epsilon_closure(nfa.initial_state),
            nfa,
            iter_c: 0,
        }
    }

    fn move_set(&self, state_set: &HashSet<u32>, input: &S) -> HashSet<u32> {
        let mut set = HashSet::new();
        for state in state_set.iter() {
            let transitions = self.nfa.transitions_from(*state);
            let input_transitions = transitions
                .into_iter()
                .filter(|(t, _)| match *t {
                    Transition::Some(symbol) => *symbol == *input,
                    Transition::Epsilon => false,
                })
                .flat_map(|(_, dest)| dest.into_iter().map(|&i| i))
                .collect();
            set = set.union(&input_transitions).map(|&i| i).collect();
        }
        set
    }
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

    #[test]
    fn test_kleene_star() {
        let c1: NFA<bool> = NFA::new_epsilon();

        let kleene = NFA::kleene_star(&c1);
        assert_eq!(4, kleene.total_states);
        assert_eq!(1, kleene.final_states.len());
    }

    #[test]
    fn test_combine() {
        let c1 = NFA::new_epsilon();
        let c2 = NFA::new_epsilon();
        let cc: Vec<&NFA<bool>> = vec![&c1, &c2];
        let combined = NFA::combine(&cc);

        assert_eq!(0, combined.initial_state);
        assert_eq!(5, combined.total_states);
        assert_eq!(2, combined.final_states.len());
    }
}

use automata::{nfa::Transition, NFA};

#[test]
fn test_new() {
    let n: NFA<bool> = NFA::new();

    assert_eq!(1, n.total_states);
    assert_eq!(0, n.start_state);
    assert_eq!(0, n.accepting_states.len());
    assert_eq!(0, n.transition.into_iter().count());
}

#[test]
fn test_new_epsilon() {
    let n: NFA<bool> = NFA::new_epsilon();

    assert_eq!(2, n.total_states);
    assert_eq!(0, n.start_state);
    assert_eq!(1, n.accepting_states.len());

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
    assert_eq!(0, n.accepting_states.len());

    let mut n: NFA<bool> = NFA::new();
    let new_state = n.add_state(true);
    assert_eq!(2, n.total_states);
    assert_eq!(n.total_states - 1, new_state);
    assert_eq!(1, n.accepting_states.len());
}

#[test]
fn test_union() {
    let c1: NFA<bool> = NFA::new_epsilon();
    let c2: NFA<bool> = NFA::new_epsilon();

    let union = NFA::union(&c1, &c2);
    assert_eq!(6, union.total_states);
    assert_eq!(1, union.accepting_states.len());
}

#[test]
fn test_concatenation() {
    let c1: NFA<bool> = NFA::new_epsilon();
    let c2: NFA<bool> = NFA::new_epsilon();

    let concat = NFA::concatenation(&c1, &c2);
    assert_eq!(4, concat.total_states);
    assert_eq!(c2.accepting_states.len(), concat.accepting_states.len());
    assert_eq!(c1.start_state, concat.start_state);
}

#[test]
fn test_kleene_star() {
    let c1: NFA<bool> = NFA::new_epsilon();

    let kleene = NFA::kleene_star(&c1);
    assert_eq!(4, kleene.total_states);
    assert_eq!(1, kleene.accepting_states.len());
}

#[test]
fn test_combine() {
    let c1 = NFA::new_epsilon();
    let c2 = NFA::new_epsilon();
    let cc: Vec<&NFA<bool>> = vec![&c1, &c2];
    let combined = NFA::combine(&cc);

    assert_eq!(0, combined.start_state);
    assert_eq!(5, combined.total_states);
    assert_eq!(2, combined.accepting_states.len());
}

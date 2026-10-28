use super::{State, Tree};
use crate::entity::{Entity, Names};

/// A small arm: base → shoulder → elbow → hand, and a sensor on the base.
fn arm() -> (Names, Tree) {
    let mut names = Names::new();
    let mut tree = Tree::new();
    let [base, shoulder, elbow, hand, sensor] = ["base", "shoulder", "elbow", "hand", "sensor"].map(|n| names.intern(n));
    assert!(tree.attach(shoulder, base));
    assert!(tree.attach(elbow, shoulder));
    assert!(tree.attach(hand, elbow));
    assert!(tree.attach(sensor, base));
    (names, tree)
}

fn e(names: &Names, n: &str) -> Entity {
    names.get(n).unwrap()
}

#[test]
fn a_tree_answers_where_a_part_hangs_and_what_moves_with_it() {
    let (names, tree) = arm();
    let [base, shoulder, elbow, hand, sensor] = ["base", "shoulder", "elbow", "hand", "sensor"].map(|n| e(&names, n));
    assert_eq!(tree.parent(hand), Some(elbow));
    assert_eq!(tree.ancestors(hand).collect::<Vec<_>>(), vec![elbow, shoulder, base]);
    assert_eq!(tree.descendants(base), vec![shoulder, elbow, hand, sensor], "depth first, in attach order");
    assert_eq!(tree.children(base), &[shoulder, sensor]);
    assert_eq!(tree.roots(), vec![base]);
    assert!(tree.is_ancestor(base, hand) && !tree.is_ancestor(hand, base));
}

#[test]
fn a_tree_refuses_a_loop_and_moves_a_subtree_whole() {
    let (names, mut tree) = arm();
    let [base, shoulder, elbow, hand, sensor] = ["base", "shoulder", "elbow", "hand", "sensor"].map(|n| e(&names, n));
    assert!(!tree.attach(base, hand), "the base cannot hang under its own hand");
    assert!(!tree.attach(elbow, elbow));
    assert_eq!(tree.parent(base), None, "a refused attach changes nothing");
    // The elbow moves to the sensor and takes the hand with it.
    assert!(tree.attach(elbow, sensor));
    assert_eq!(tree.children(shoulder), &[] as &[Entity]);
    assert_eq!(tree.ancestors(hand).collect::<Vec<_>>(), vec![elbow, sensor, base]);
    assert_eq!(tree.detach(elbow), Some(sensor));
    assert_eq!(tree.roots(), vec![base, elbow], "a detached subtree is a tree of its own");
}

#[test]
fn state_reads_the_last_step_until_the_next_is_committed() {
    let mut names = Names::new();
    let [a, b] = ["a", "b"].map(|n| names.intern(n));
    let mut qpos: State<f64> = State::new();
    qpos.init(a, 0.0);
    qpos.init(b, 10.0);
    // A step: a moves, b is not written.
    qpos.set(a, 1.0);
    assert_eq!(qpos.get(a), Some(&0.0), "a read during the step sees the last step");
    assert_eq!(qpos.pending(a), Some(&1.0));
    qpos.commit();
    assert_eq!((qpos.get(a), qpos.get(b), qpos.step()), (Some(&1.0), Some(&10.0), 1));
    qpos.set(b, 11.0);
    qpos.commit();
    assert_eq!((qpos.get(a), qpos.get(b), qpos.step()), (Some(&1.0), Some(&11.0), 2));
    assert_eq!(qpos.get(names.intern("c")), None);
}

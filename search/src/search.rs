use rand::{Rng, RngExt};

use crate::dag::MultiDag;
use crate::enumerate::get_random_tree;
use crate::tree::Tree;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct DagInvariant {
    node_count: usize,
    edges_count: usize,
    edges_count_single: usize,
    finals: usize,
}

impl DagInvariant {
    fn of(d: &MultiDag<Tree>) -> Self {
        Self {
            node_count: d.node_count(),
            edges_count: d.edges_count(),
            edges_count_single: d.edges_count_single(),
            finals: d.get_finals().len(),
        }
    }
}

fn are_dyslexic_rec(t: &Tree, t2: &Tree, height: usize, choices: &mut Vec<Option<bool>>) -> bool {
    if t.children.len() != t2.children.len() {
        return false;
    }
    if t.is_leaf() {
        return true;
    }

    if choices.len() <= height {
        choices.resize(height + 1, None);
    }

    let try_with = |reverse_here: bool, choices: &mut Vec<Option<bool>>| -> bool {
        let iter2: Box<dyn Iterator<Item = &Tree>> = if reverse_here {
            Box::new(t2.children.iter().rev())
        } else {
            Box::new(t2.children.iter())
        };
        t.children
            .iter()
            .zip(iter2)
            .all(|(a, b)| are_dyslexic_rec(a, b, height + 1, choices))
    };

    match choices[height] {
        Some(reverse_here) => try_with(reverse_here, choices),
        None => {
            let checkpoint = choices.len();

            choices[height] = Some(false);
            if try_with(false, choices) {
                return true;
            }
            choices.truncate(checkpoint);

            if choices.len() <= height {
                choices.resize(height + 1, None);
            }

            choices[height] = Some(true);
            if try_with(true, choices) {
                return true;
            }
            choices.truncate(checkpoint);
            if choices.len() <= height {
                choices.resize(height + 1, None);
            }
            choices[height] = None;
            false
        }
    }
}

/// Returns true iff there exists a set `P ⊆ ℕ` such that `ρ_P^0(t) = t2`.
///
/// Here `P` is a *global* set of heights: for all nodes at the same height, the child order is
/// either always kept or always reversed.
pub fn are_dyslexic(t: &Tree, t2: &Tree) -> bool {
    are_dyslexic_rec(t, t2, 0, &mut Vec::new())
}

/// Apply `ρ_P^0` to a tree, where `P` is encoded by a predicate over heights.
pub fn rho_by_height(t: &Tree, height: usize, in_p: &impl Fn(usize) -> bool) -> Tree {
    if t.is_leaf() {
        return Tree::leaf();
    }
    let mut children: Vec<Tree> = t
        .children
        .iter()
        .map(|c| rho_by_height(c, height + 1, in_p))
        .collect();
    if in_p(height) {
        children.reverse();
    }
    Tree { children }
}

/// Generates a (possibly big) dyslexic pair `(t, ρ_P^0(t))` by sampling a random tree and a random
/// set `P` (chosen independently per height up to `max_height`).
pub fn generate_dyslexic_pair<R: Rng + ?Sized>(
    rng: &mut R,
    edge_count: usize,
    max_height: usize,
) -> (Tree, Tree) {
    let t = get_random_tree(rng, edge_count);
    let p: Vec<bool> = (0..=max_height).map(|_| rng.random_bool(0.5)).collect();
    let t2 = rho_by_height(&t, 0, &|h| p.get(h).copied().unwrap_or(false));
    (t, t2)
}

/// Tries to generate a pair of (big) trees that have the same reduction DAG
/// by searching for collisions in a random pool.
pub fn generate_same_dag_pair_by_collision<R: Rng + ?Sized>(
    rng: &mut R,
    edge_count: usize,
    pool: usize,
    bucket_limit: usize,
) -> Option<(Tree, Tree)> {
    use std::collections::HashMap;

    let mut buckets: HashMap<DagInvariant, Vec<(Tree, MultiDag<Tree>)>> = HashMap::new();

    for _ in 0..pool {
        let t = get_random_tree(rng, edge_count);
        let d = MultiDag::from_tree(&t);
        let inv = DagInvariant::of(&d);

        let entry = buckets.entry(inv).or_default();
        for (prev_t, prev_d) in entry.iter() {
            if prev_d == &d {
                return Some((prev_t.clone(), t));
            }
        }

        if entry.len() < bucket_limit {
            entry.push((t, d));
        }
    }

    None
}

/// Searches for a counterexample to the implication:
/// `MultiDag::from_tree(t) == MultiDag::from_tree(t2)  =>  are_dyslexic(t,t2)`.
///
/// Returns `Some((t, t2))` iff a DAG-equal but non-dyslexic pair is found.
pub fn find_dag_equivalent_not_dyslexic_counterexample<R: Rng + ?Sized>(
    rng: &mut R,
    edge_count: usize,
    pool: usize,
    bucket_limit: usize,
) -> Option<(Tree, Tree)> {
    use std::collections::HashMap;

    let mut buckets: HashMap<DagInvariant, Vec<(Tree, MultiDag<Tree>)>> = HashMap::new();

    for _ in 0..pool {
        let t = get_random_tree(rng, edge_count);
        let d = MultiDag::from_tree(&t);
        let inv = DagInvariant::of(&d);

        let entry = buckets.entry(inv).or_default();
        for (prev_t, prev_d) in entry.iter() {
            if prev_d == &d && !are_dyslexic(prev_t, &t) {
                return Some((prev_t.clone(), t));
            }
        }

        if entry.len() < bucket_limit {
            entry.push((t, d));
        }
    }

    None
}

/// Searches for a counterexample to the conjecture:
/// `t ~ t2  <=>  dag::from_tree(t) == dag::from_tree(t2)`.
///
/// Returns `Some((t, t2))` if a counterexample is found, otherwise `None`.
pub fn find_dyslexic_dag_conjecture_counterexample<R: Rng + ?Sized>(
    rng: &mut R,
    edge_count: usize,
    trials: usize,
) -> Option<(Tree, Tree)> {
    for _ in 0..trials {
        let t = get_random_tree(rng, edge_count);
        let t2 = get_random_tree(rng, edge_count);

        let dys = are_dyslexic(&t, &t2);
        let dag_eq = MultiDag::from_tree(&t) == MultiDag::from_tree(&t2);

        if dys != dag_eq {
            return Some((t, t2));
        }
    }
    None
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dyslexic_reflexive() {
        let t: Tree = "(()(()))".parse().unwrap();
        assert!(are_dyslexic(&t, &t));
    }

    #[test]
    fn dyslexic_reverse_at_root() {
        let t: Tree = "(()(()))".parse().unwrap();
        let t2: Tree = "((())())".parse().unwrap();
        assert!(are_dyslexic(&t, &t2));
        assert!(are_dyslexic(&t2, &t));
    }

    #[test]
    fn dyslexic_distinguishes_non_examples() {
        let t: Tree = "((())(()))".parse().unwrap();
        let t2: Tree = "((())())".parse().unwrap();
        assert!(!are_dyslexic(&t, &t2));
    }
    
    #[test]
    fn constant_at_height() {
        let t1: Tree = "(   ( ( ()(()) ) () )  ( (()(())) () )   )".parse().unwrap();
        let t2: Tree = "(   ( ( ()(()) ) () )  ( ((())()) () )   )".parse().unwrap();

        assert!(!are_dyslexic(&t1,&t2));
    }

    #[test]
    fn rho_produces_dyslexic_pair() {
        let t: Tree = "((()(()))())".parse().unwrap();
        let t2 = rho_by_height(&t, 0, &|h| h == 0);
        assert!(are_dyslexic(&t, &t2));
    }

    #[test]
    fn dag_implies_dyslexic_search_smoke() {
        let mut rng = rand::rng();
        let _ = find_dag_equivalent_not_dyslexic_counterexample(&mut rng, 9, 200, 20);
    }

    #[test]
    fn conjecture_finds_counterexample_quickly() {
        let mut rng = rand::rng();
        // This test is only here to exercise the code path (and ensure it doesn't panic).
        let _ = find_dyslexic_dag_conjecture_counterexample(&mut rng, 10, 30);
    }
}

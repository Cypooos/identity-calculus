use std::collections::BTreeMap;

use rand::Rng;

use crate::dag::MultiDag;
use crate::enumerate::get_random_tree;
use crate::tree::Tree;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HeightRule {
    Keep,
    Reverse,
}

fn compare_with_rule(t: &Tree, t2: &Tree, rule: HeightRule) -> bool {
    if t.children.len() != t2.children.len() {
        return false;
    }
    let left = t.children.iter();
    let right: Box<dyn Iterator<Item = &Tree>> = match rule {
        HeightRule::Keep => Box::new(t2.children.iter()),
        HeightRule::Reverse => Box::new(t2.children.iter().rev()),
    };
    left.zip(right).all(|(a, b)| a == b)
}

fn are_dyslexic_rec(
    t: &Tree,
    t2: &Tree,
    height: usize,
    rules: &BTreeMap<usize, HeightRule>,
) -> bool {
    if t.children.len() != t2.children.len() {
        return false;
    }

    if t.is_leaf() {
        return true;
    }

    let known = rules.get(&height).copied();

    let try_orientation = |rule: HeightRule, mut next_rules: BTreeMap<usize, HeightRule>| {
        next_rules.insert(height, rule);
        let iter2: Box<dyn Iterator<Item = &Tree>> = match rule {
            HeightRule::Keep => Box::new(t2.children.iter()),
            HeightRule::Reverse => Box::new(t2.children.iter().rev()),
        };
        t.children
            .iter()
            .zip(iter2)
            .all(|(a, b)| are_dyslexic_rec(a, b, height + 1, &next_rules))
    };

    match known {
        Some(rule) => {
            if !compare_with_rule(t, t2, rule) {
                return false;
            }
            let iter2: Box<dyn Iterator<Item = &Tree>> = match rule {
                HeightRule::Keep => Box::new(t2.children.iter()),
                HeightRule::Reverse => Box::new(t2.children.iter().rev()),
            };
            t.children
                .iter()
                .zip(iter2)
                .all(|(a, b)| are_dyslexic_rec(a, b, height + 1, rules))
        }
        None => {
            // Branch: decide whether this height is in P or not.
            // Early-prune: if neither orientation can match immediate children counts, fail.
            try_orientation(HeightRule::Keep, rules.clone())
                || try_orientation(HeightRule::Reverse, rules.clone())
        }
    }
}

/// Returns true iff there exists a set `P ⊆ ℕ` such that `ρ_P^0(t) = t2`.
///
/// Here `P` is a *global* set of heights: for all nodes at the same height, the child order is
/// either always kept or always reversed.
pub fn are_dyslexic(t: &Tree, t2: &Tree) -> bool {
    are_dyslexic_rec(t, t2, 0, &BTreeMap::new())
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
        let dag_eq = dag_equivalent_fixed(&MultiDag::from_tree(&t), &MultiDag::from_tree(&t2));

        if dys != dag_eq {
            return Some((t, t2));
        }
    }
    None
}

fn dag_equivalent_fixed(left: &MultiDag<Tree>, right: &MultiDag<Tree>) -> bool {
    use itertools::Itertools;
    use std::collections::HashMap;

    fn rec(
        left: &MultiDag<Tree>,
        left_node: usize,
        right: &MultiDag<Tree>,
        right_node: usize,
        memo: &mut HashMap<(usize, usize), bool>,
    ) -> bool {
        if let Some(&v) = memo.get(&(left_node, right_node)) {
            return v;
        }

        let left_lbl = &left.nodes[left_node].label;
        let right_lbl = &right.nodes[right_node].label;

        let res = if left_lbl == right_lbl {
            true
        } else {
            let left_children: Vec<(usize, Vec<usize>)> = left
                .children(left_node)
                .into_iter()
                .sorted_by_key(|&(_, ar)| ar)
                .chunk_by(|&(_, ar)| ar)
                .into_iter()
                .map(|(ar, group)| (ar, group.map(|(n, _)| n).collect()))
                .collect();

            let right_children: Vec<(usize, Vec<usize>)> = right
                .children(right_node)
                .into_iter()
                .sorted_by_key(|&(_, ar)| ar)
                .chunk_by(|&(_, ar)| ar)
                .into_iter()
                .map(|(ar, group)| (ar, group.map(|(n, _)| n).collect()))
                .collect();

            if left_children.len() != right_children.len()
                || left_children.iter().zip(&right_children).any(|((ar1, v1), (ar2, v2))| {
                    *ar1 != *ar2 || v1.len() != v2.len()
                })
            {
                false
            } else {
                left_children
                    .into_iter()
                    .zip(right_children)
                    .all(|((_, lnodes), (_, rnodes))| {
                        lnodes
                            .iter()
                            .permutations(lnodes.len())
                            .any(|perm| {
                                perm.into_iter()
                                    .zip(rnodes.iter())
                                    .all(|(ln, rn)| rec(left, *ln, right, *rn, memo))
                            })
                    })
            }
        };

        memo.insert((left_node, right_node), res);
        res
    }

    rec(left, left.root, right, right.root, &mut HashMap::new())
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
    fn conjecture_finds_counterexample_quickly() {
        let mut rng = rand::rng();
        // This test is only here to exercise the code path (and ensure it doesn't panic).
        let _ = find_dyslexic_dag_conjecture_counterexample(&mut rng, 7, 200);
    }
}

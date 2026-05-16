
use std::collections::{HashMap, HashSet};

use crate::immediate_reductions;
use crate::parse::parse_tree;
use crate::tree::Tree;

use rand::{Rng, RngExt};

pub fn catalan(n: usize) -> u128 {
    let mut c: u128 = 1; // C_0 = 1

    for k in 0..n {
        c = c * (4 * k as u128 + 2) / (k as u128 + 2);
    }

    c
}

fn get_dyck_words(pairs: usize) -> Vec<String> {
    fn generate(
        pairs: usize,
        memo: &mut std::collections::HashMap<usize, Vec<String>>,
    ) -> Vec<String> {
        if let Some(v) = memo.get(&pairs) {
            return v.clone();
        }
        let out = if pairs == 0 {
            vec![String::new()]
        } else {
            let mut out = Vec::new();
            for k in 0..pairs {
                let left = generate(k, memo);
                let right = generate(pairs - 1 - k, memo);
                for l in &left {
                    for r in &right {
                        out.push(format!("({l}){r}"));
                    }
                }
            }
            out
        };
        memo.insert(pairs, out.clone());
        out
    }

    generate(pairs, &mut std::collections::HashMap::new())
}

/// Enumerate all trees with edge_count edges
pub fn get_trees(edge_count: usize) -> Vec<Tree> {
        let inner = get_dyck_words(edge_count);
        inner.into_iter().map(|w| format!("({w})")).map(|s|parse_tree(&s).unwrap()).collect()
}

/// Give a random generated tree with edge_count edges
pub fn get_random_tree<R: Rng + ?Sized>(rng: &mut R, edge_count: usize) -> Tree {
    let mut children: Vec<Vec<usize>> = vec![Vec::new()];

    for new_node in 1..=edge_count {
        let parent = rng.random_range(0..new_node);
        children.push(Vec::new());
        children[parent].push(new_node);
    }

    fn build(node: usize, children: &[Vec<usize>]) -> Tree {
        Tree {
            children: children[node]
                .iter()
                .map(|&child| build(child, children))
                .collect(),
        }
    }

    build(0, &children)
}

/// Get number of reduction from tree to I, with or without multiplicity
pub fn get_nb_reductions(tree:Tree, multi:bool) -> HashMap<Tree,usize> {

    fn count(tree:&Tree, memo:&mut HashMap<Tree,usize>, multi:bool) {
        if memo.contains_key(&tree) {return;};
        if tree.is_leaf() {
            memo.insert(Tree::leaf(), 1);
            return
        };
        let mut reduct = immediate_reductions(&tree); // with multiplicity here
        if !multi {
            let mut set = HashSet::new();
            set.extend(reduct.into_iter());
            reduct = set.into_iter().collect();
        };
        let mut res = 0;
        for red in reduct {
            count(&red, memo,multi);
            res += memo.get(&red).unwrap();
        }
        memo.insert(tree.clone(), res);
    }

    let mut res = HashMap::new();
    count(&tree,&mut res,multi);
    res
}

/// Get number of reduction from tree to I with multiplicity
/// Denote N(T) the number of nodes in T with V set of non-root vertexes, the formula is (N(T)-1)!/(Prod_(x in V) N(x))
pub fn get_hook_length(tree:&Tree) -> usize {
    let n = tree.edge_count();
    let fact = (1..=n).fold(1,|x,y|x*y);
    let mut deno = 1;
    tree.for_each_strict(&mut |x|{deno = deno*x.node_count()});
    fact/deno
}



#[cfg(test)]
mod test {

    #[test]
    fn enumerate_by_edges_matches_small_counts() {
        //assert_eq!(get_trees(0), vec!["()"]);
        //assert_eq!(get_trees(1), vec!["(())"]);
        //let e2 = get_trees(2);
        //assert_eq!(e2, vec!["((()))", "(()())"]);
    }
}
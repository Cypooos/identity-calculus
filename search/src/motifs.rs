use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::reduction::immediate_reductions;
use crate::tree::Tree;

/// Témoin d'un motif "diamond3" sur 2 pas : `top -> mid_i -> bottom` (i=1..3).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diamond3Witness {
    pub top: String,
    pub middle: [String; 3],
    pub bottom: String,
}

/// Cherche un motif "diamond3" (3 successeurs distincts qui convergent en 2 pas).
///
/// On considère des arêtes simples (on ignore les multiplicités) et des nœuds distincts
/// (fusionnés par forme canonique).
pub fn find_diamond3(top: &Tree) -> Option<Diamond3Witness> {
    let top_canon = top.to_string();

    let mut middles: BTreeMap<String, Tree> = BTreeMap::new();
    for t in immediate_reductions(top) {
        middles.entry(t.to_string()).or_insert(t);
    }
    if middles.len() < 3 {
        return None;
    }

    let mut bottom_to_middles: HashMap<String, BTreeSet<String>> = HashMap::new();
    for (mid_canon, mid_tree) in &middles {
        let mut bottoms: BTreeSet<String> = BTreeSet::new();
        for b in immediate_reductions(mid_tree) {
            bottoms.insert(b.to_string());
        }
        for bottom in bottoms {
            bottom_to_middles
                .entry(bottom)
                .or_default()
                .insert(mid_canon.clone());
        }
    }

    let mut bottoms: Vec<_> = bottom_to_middles.into_iter().collect();
    bottoms.sort_by(|(a, _), (b, _)| a.cmp(b));
    for (bottom, mids) in bottoms {
        if mids.len() >= 3 {
            let mut mids: Vec<String> = mids.into_iter().collect();
            mids.sort();
            return Some(Diamond3Witness {
                top: top_canon,
                middle: [mids[0].clone(), mids[1].clone(), mids[2].clone()],
                bottom,
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_diamond3_on_small_example() {
        let t: Tree = "(()())".parse().unwrap();
        assert_eq!(find_diamond3(&t), None);
    }
}


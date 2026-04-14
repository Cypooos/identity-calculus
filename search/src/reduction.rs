use crate::tree::Tree;

/// Retourne toutes les réductions immédiates (suppression d'une feuille non-racine).
///
/// Important : si deux suppressions différentes donnent le même arbre résultat, le résultat
/// contient plusieurs occurrences (utile pour les multi-arêtes).
pub fn immediate_reductions(tree: &Tree) -> Vec<Tree> {
    tree.non_root_leaf_paths()
        .into_iter()
        .filter_map(|p| tree.delete_leaf(&p).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reductions_keep_multiplicity() {
        let t: Tree = "(()())".parse().unwrap();
        let reductions = immediate_reductions(&t);
        assert_eq!(reductions.len(), 2);
        assert_eq!(reductions[0].to_string(), "(())");
        assert_eq!(reductions[1].to_string(), "(())");
    }
}

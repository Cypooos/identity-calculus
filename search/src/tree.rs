use std::fmt;

use thiserror::Error;


/// Arbre enraciné non étiqueté, avec enfants ordonnés.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Tree {
    pub children: Vec<Tree>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TreeEditError {
    #[error("path is empty (cannot delete the root)")]
    EmptyPath,
    #[error("invalid child index {index} at depth {depth}")]
    InvalidIndex { depth: usize, index: usize },
    #[error("target is not a leaf")]
    TargetNotLeaf,
}

impl Tree {
    /// Create a leaf ()
    pub fn leaf() -> Self {
        Self { children: vec![] }
    }

    /// Return if the tree is a leaf
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// Number of nodes
    pub fn node_count(&self) -> usize {
        1 + self.children.iter().map(|c| c.node_count()).sum::<usize>()
    }

    /// Numbre of edges
    pub fn edge_count(&self) -> usize {
        self.children.len() + self.children.iter().map(|c| c.edge_count()).sum::<usize>()
    }

    /// Liste tous les chemins vers des feuilles non-racines.
    ///
    /// Un chemin est une suite d'indices d'enfants (depuis la racine).
    pub fn non_root_leaf_paths(&self) -> Vec<Vec<usize>> {
        let mut out = Vec::new();
        let mut prefix = Vec::new();
        self.collect_non_root_leaf_paths(&mut prefix, &mut out);
        out
    }

    fn collect_non_root_leaf_paths(&self, prefix: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        for (i, child) in self.children.iter().enumerate() {
            prefix.push(i);
            if child.is_leaf() {
                out.push(prefix.clone());
            } else {
                child.collect_non_root_leaf_paths(prefix, out);
            }
            prefix.pop();
        }
    }

    /// Supprime une feuille non-racine, donnée par un chemin, et retourne le nouvel arbre.
    pub fn delete_leaf(&self, path: &[usize]) -> Result<Tree, TreeEditError> {
        if path.is_empty() {
            return Err(TreeEditError::EmptyPath);
        }
        let mut new = self.clone();
        new.delete_leaf_in_place(path, 0)?;
        Ok(new)
    }

    fn delete_leaf_in_place(
        &mut self,
        path: &[usize],
        depth: usize,
    ) -> Result<(), TreeEditError> {
        let Some((&first, rest)) = path.split_first() else {
            return Err(TreeEditError::EmptyPath);
        };
        if first >= self.children.len() {
            return Err(TreeEditError::InvalidIndex {
                depth,
                index: first,
            });
        }

        if rest.is_empty() {
            if !self.children[first].is_leaf() {
                return Err(TreeEditError::TargetNotLeaf);
            }
            self.children.remove(first);
            return Ok(());
        }

        self.children[first].delete_leaf_in_place(rest, depth + 1)
    }

    pub fn for_each<T:FnMut(&Tree) -> ()>(&self, f :&mut T) {
        f(self);
        self.children.iter().for_each(|x|x.for_each(f));
    }

    pub fn for_each_strict<T:FnMut(&Tree) -> ()>(&self, f :&mut T) {
        self.children.iter().for_each(|x|x.for_each(f));
    }

    pub fn to_canonical(&mut self) {
        self.children.iter_mut().for_each(|x|x.to_canonical());
        self.children.sort();
    }
    // ( () (()) )
    // () ,  (())
    // (()), ()

    //
    pub fn to_i(&self) -> String {
        return self.children.iter().rev().fold(format!("I"), |s,c|format!("({} {s})", c.to_i()))
    }

}

// l1 = ((I ((I I) I)) ((I ((I I) I)) I))
//      
// l2 = ((I ((I I) I)) (((I I) (I I)) I))
//      


impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("(")?;
        for child in &self.children {
            write!(f, "{child}")?;
        }
        f.write_str(")")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_is_canonical() {
        let t: Tree = " ( () (()) ) ".parse().unwrap();
        assert_eq!(t.to_string(), "(()(()))");
    }

    #[test]
    fn parse_examples() {
        let leaf: Tree = "()".parse().unwrap();
        assert!(leaf.is_leaf());
        assert_eq!(leaf.node_count(), 1);
        assert_eq!(leaf.edge_count(), 0);

        let t: Tree = "(())".parse().unwrap();
        assert_eq!(t.edge_count(), 1);

        let t: Tree = "(()())".parse().unwrap();
        assert_eq!(t.edge_count(), 2);
    }

    #[test]
    fn leaf_paths_and_delete() {
        let t: Tree = "(()())".parse().unwrap();
        let paths = t.non_root_leaf_paths();
        assert_eq!(paths, vec![vec![0], vec![1]]);
        assert_eq!(t.delete_leaf(&paths[0]).unwrap().to_string(), "(())");
        assert_eq!(t.delete_leaf(&paths[1]).unwrap().to_string(), "(())");
    }

}

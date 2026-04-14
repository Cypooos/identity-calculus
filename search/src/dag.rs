use std::collections::{HashSet, VecDeque};

use crate::reduction::immediate_reductions;
use crate::tree::Tree;

pub type NodeId = usize;

#[derive(Clone, Debug)]
pub struct DagNode<T> {
    pub id: NodeId,
    pub canon: String,
    pub edge_count: usize,
    pub label: Option<T>,
}

#[derive(Clone, Debug)]
pub struct MultiDag<T = Tree> {
    pub nodes: Vec<DagNode<T>>,
    pub canon_to_id: std::collections::HashMap<String, NodeId>,
    pub edges: std::collections::HashMap<(NodeId, NodeId), usize>, // multiplicité
    pub levels: Vec<Vec<NodeId>>, // indexé par nb d’arêtes
    pub root: NodeId,
}

impl<T> Default for MultiDag<T> {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            canon_to_id: std::collections::HashMap::new(),
            edges: std::collections::HashMap::new(),
            levels: Vec::new(),
            root: 0,
        }
    }
}

impl MultiDag<Tree> {
    /// Construit le DAG de réduction (avec multi-arêtes) depuis un arbre racine.
    ///
    /// Les sommets sont fusionnés par forme canonique (affichage parenthésé sans espaces).
    pub fn from_tree(root: &Tree, keep_labels: bool) -> Self {
        let root_edge_count = root.edge_count();

        let mut dag = MultiDag {
            levels: vec![Vec::new(); root_edge_count + 1],
            ..Default::default()
        };

        let root_id = dag.intern_node(root, keep_labels);
        dag.root = root_id;

        let mut queue = VecDeque::new();
        let mut expanded = HashSet::<NodeId>::new();
        queue.push_back((root_id, root.clone()));

        while let Some((from_id, from_tree)) = queue.pop_front() {
            if !expanded.insert(from_id) {
                continue;
            }
            for succ_tree in immediate_reductions(&from_tree) {
                let to_id = dag.intern_node(&succ_tree, keep_labels);
                *dag.edges.entry((from_id, to_id)).or_insert(0) += 1;
                if !expanded.contains(&to_id) {
                    queue.push_back((to_id, succ_tree));
                }
            }
        }

        dag
    }

    fn intern_node(&mut self, tree: &Tree, keep_labels: bool) -> NodeId {
        let canon = tree.to_string();
        let edge_count = tree.edge_count();
        let label = keep_labels.then(|| tree.clone());
        self.intern_node_parts(canon, edge_count, label)
    }
}

impl<T> MultiDag<T> {
    /// Arêtes distinctes (vue "simple", sans multiplicité).
    pub fn distinct_edges(&self) -> Vec<(NodeId, NodeId)> {
        let mut out: Vec<_> = self.edges.keys().copied().collect();
        out.sort_unstable();
        out
    }

    /// Nombre d'arêtes distinctes.
    pub fn distinct_edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Somme des multiplicités de toutes les arêtes.
    pub fn total_multiplicity(&self) -> usize {
        self.edges.values().copied().sum()
    }

    /// Applique `f` à tous les labels et retourne un nouveau `MultiDag`.
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> MultiDag<U> {
        MultiDag {
            nodes: self
                .nodes
                .into_iter()
                .map(|n| DagNode {
                    id: n.id,
                    canon: n.canon,
                    edge_count: n.edge_count,
                    label: n.label.map(&mut f),
                })
                .collect(),
            canon_to_id: self.canon_to_id,
            edges: self.edges,
            levels: self.levels,
            root: self.root,
        }
    }

    /// Supprime les labels (équivalent à `map(|_| ())`).
    pub fn remove_label(self) -> MultiDag<()> {
        self.map(|_| ())
    }

    fn intern_node_parts(&mut self, canon: String, edge_count: usize, label: Option<T>) -> NodeId {
        if let Some(&id) = self.canon_to_id.get(&canon) {
            if self.nodes[id].label.is_none() {
                self.nodes[id].label = label;
            }
            return id;
        }

        let id = self.nodes.len();
        self.nodes.push(DagNode {
            id,
            canon: canon.clone(),
            edge_count,
            label,
        });
        self.canon_to_id.insert(canon, id);
        self.levels[edge_count].push(id);
        id
    }

    /// Résumé : tailles des niveaux indexées par nb d'arêtes.
    pub fn level_sizes(&self) -> Vec<usize> {
        self.levels.iter().map(|lvl| lvl.len()).collect()
    }

    /// Nombre de sommets.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Enfants d'un sommet, avec multiplicité des arcs `(to, mult)`.
    pub fn children(&self, from: NodeId) -> Vec<(NodeId, usize)> {
        let mut out: Vec<(NodeId, usize)> = self
            .edges
            .iter()
            .filter_map(|((f, t), m)| (*f == from).then_some((*t, *m)))
            .collect();
        out.sort_unstable_by_key(|(to, _)| *to);
        out
    }

    /// Parents d'un sommet, avec multiplicité des arcs `(from, mult)`.
    pub fn parents(&self, to: NodeId) -> Vec<(NodeId, usize)> {
        let mut out: Vec<(NodeId, usize)> = self
            .edges
            .iter()
            .filter_map(|((f, t), m)| (*t == to).then_some((*f, *m)))
            .collect();
        out.sort_unstable_by_key(|(from, _)| *from);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dag_merges_nodes_and_counts_multiedges() {
        let t: Tree = "(()())".parse().unwrap();
        let dag = MultiDag::from_tree(&t, true);
        assert_eq!(dag.node_count(), 3);
        assert_eq!(dag.distinct_edge_count(), 2);
        assert_eq!(dag.total_multiplicity(), 3);

        let root = dag.root;
        let mid = dag
            .nodes
            .iter()
            .find(|n| n.edge_count == 1)
            .unwrap()
            .id;
        assert_eq!(dag.edges.get(&(root, mid)).copied(), Some(2));

        assert_eq!(dag.children(root), vec![(mid, 2)]);
        assert_eq!(dag.parents(mid), vec![(root, 2)]);
    }

    #[test]
    fn map_and_remove_label() {
        let t: Tree = "(()())".parse().unwrap();
        let dag = MultiDag::from_tree(&t, true);
        let dag2: MultiDag<bool> = dag.clone().map(|label| label.edge_count() == 2);
        assert_eq!(dag2.node_count(), dag.node_count());
        assert!(dag2.nodes.iter().any(|n| n.label == Some(true)));

        let dag3 = dag.remove_label();
        assert!(dag3.nodes.iter().any(|n| n.label == Some(())));
    }
}

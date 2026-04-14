use std::collections::{HashSet, VecDeque};

use crate::reduction::immediate_reductions;
use crate::tree::Tree;

pub type NodeId = usize;

#[derive(Clone, Debug)]
pub struct DagNode {
    pub id: NodeId,
    pub canon: String,
    pub edge_count: usize,
    pub tree: Option<Tree>, // présent si version labellisée
}

#[derive(Clone, Debug, Default)]
pub struct MultiDag {
    pub nodes: Vec<DagNode>,
    pub canon_to_id: std::collections::HashMap<String, NodeId>,
    pub edges: std::collections::HashMap<(NodeId, NodeId), usize>, // multiplicité
    pub levels: Vec<Vec<NodeId>>, // indexé par nb d’arêtes
    pub root: NodeId,
}

impl MultiDag {
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
                if dag.nodes[to_id].tree.is_none() && !keep_labels {
                    // no-op: this just documents that we keep the structure for discovery only
                }
                if !expanded.contains(&to_id) {
                    queue.push_back((to_id, succ_tree));
                }
            }
        }

        dag
    }

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

    fn intern_node(&mut self, tree: &Tree, keep_labels: bool) -> NodeId {
        let canon = tree.to_string();
        if let Some(&id) = self.canon_to_id.get(&canon) {
            return id;
        }

        let id = self.nodes.len();
        let edge_count = tree.edge_count();
        self.nodes.push(DagNode {
            id,
            canon: canon.clone(),
            edge_count,
            tree: keep_labels.then(|| tree.clone()),
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
    }
}

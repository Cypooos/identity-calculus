use std::fmt;

use crate::dag::MultiDag;
use crate::tree::Tree;

/// Wrapper label + information de couleur pour DOT.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DotColor<T> {
    pub value: T,
    pub red: bool,
}

/// Représentation d'un label dans DOT (texte + éventuel style).
pub trait DotLabel {
    fn dot_text(&self) -> String;
    fn dot_is_red(&self) -> bool {
        false
    }
}

impl<T: fmt::Display> fmt::Display for DotColor<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<T: fmt::Display> DotLabel for DotColor<T> {
    fn dot_text(&self) -> String {
        self.value.to_string()
    }
    fn dot_is_red(&self) -> bool {
        self.red
    }
}

impl DotLabel for Tree {
    fn dot_text(&self) -> String {
        self.to_string()
    }
}

impl DotLabel for bool {
    fn dot_text(&self) -> String {
        self.to_string()
    }
    fn dot_is_red(&self) -> bool {
        *self
    }
}

impl DotLabel for (Tree, bool) {
    fn dot_text(&self) -> String {
        self.0.to_string()
    }
    fn dot_is_red(&self) -> bool {
        self.1
    }
}

/// Exporte un `MultiDag` au format Graphviz DOT.
///
/// - Si un sommet a un `label`, il est affiché (via `DotLabel::dot_text`).
/// - Sinon, on affiche la forme canonique.
/// - Si le label est "marqué" (`dot_is_red`), le sommet est coloré en rouge.
pub fn to_dot<T: DotLabel>(dag: &MultiDag<T>, show_edge_multiplicity: bool) -> String {
    let mut s = String::new();
    s.push_str("digraph G {\n");
    s.push_str("  rankdir=TB;\n");
    s.push_str("  node [shape=box];\n");

    for node in &dag.nodes {
        let label_text = match &node.label {
            Some(l) => l.dot_text(),
            None => node.canon.clone(),
        };
        let is_red = node.label.as_ref().is_some_and(|l| l.dot_is_red());
        if is_red {
            s.push_str(&format!(
                "  n{} [label=\"{}\", style=filled, fillcolor=red, fontcolor=white];\n",
                node.id,
                escape_dot(&label_text)
            ));
        } else {
            s.push_str(&format!(
                "  n{} [label=\"{}\"];\n",
                node.id,
                escape_dot(&label_text)
            ));
        }
    }

    let mut edges: Vec<_> = dag.edges.iter().collect();
    edges.sort_by_key(|((from, to), _)| (*from, *to));
    for ((from, to), mult) in edges {
        if show_edge_multiplicity && *mult != 1 {
            s.push_str(&format!(
                "  n{} -> n{} [label=\"{}\"];\n",
                from, to, mult
            ));
        } else {
            s.push_str(&format!("  n{} -> n{};\n", from, to));
        }
    }

    s.push_str("}\n");
    s
}

fn escape_dot(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dot_contains_nodes_and_edges() {
        let t: Tree = "(()())".parse().unwrap();
        let dag = MultiDag::from_tree(&t, true);
        let dot = to_dot(&dag, true);
        assert!(dot.contains("digraph G"));
        assert!(dot.contains("->"));
        assert!(dot.contains("label=\"2\""));
    }

    #[test]
    fn dot_colors_bool_true_as_red() {
        let t: Tree = "(()())".parse().unwrap();
        let dag = MultiDag::from_tree(&t, true).map(|tree| tree.edge_count() == 2);
        let dot = to_dot(&dag, false);
        assert!(dot.contains("fillcolor=red"));
    }
}

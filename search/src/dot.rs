use itertools::Itertools;

use crate::dag::MultiDag;
use crate::tree::Tree;

/// Représentation d'un label dans DOT (texte + éventuel style).
pub trait DotLabel {
    fn get_text(&self) -> String {format!("")}
    fn get_meta(&self) -> String {format!("")}
}

impl DotLabel for Tree {
    fn get_text(&self) -> String { format!("{}",self) }
}

impl DotLabel for bool {
    fn get_meta(&self) -> String { if *self {format!("color=red")} else {format!("")} }
}

impl<T,U> DotLabel for (T, U) where T:DotLabel, U:DotLabel {
    fn get_text(&self) -> String { self.0.get_text() }
    fn get_meta(&self) -> String { self.1.get_meta() }
}

/// Exporte un `MultiDag` au format Graphviz DOT.
///
/// - Si un sommet a un `label`, il est affiché (via `DotLabel::dot_text`).
/// - Sinon, on affiche la forme canonique.
/// - Si le label est "marqué" (`dot_is_red`), le sommet est coloré en rouge.
pub fn to_dot<T: DotLabel>(dag: &MultiDag<T>, show_mult: bool) -> String {
    let mut s = String::new();
    s.push_str("digraph G {\n");
    s.push_str("  rankdir=TB;\n");
    s.push_str("  node [shape=box];\n");

    for node in &dag.nodes {
        let label_text = node.label.get_text();
        let params = node.label.get_meta();
        s.push_str(&format!(
            "  n{} [label=\"{}\", {}];\n",
            node.id,
            escape_dot(&label_text),
            escape_dot(&params),
        ));
    }

    for (from, children) in dag.edges.iter() {
        children
            .iter()
            .sorted()
            .chunk_by(|e|*e)
            .into_iter()
            .map(|(k,group)|(*k,group.count()))
            .for_each(|(to,mult)| {
                if show_mult && mult > 1 {
                    s.push_str(&format!(
                    "  n{} -> n{} [label=\"{}\"];\n",
                    from, to, mult));
                } else {
                    s.push_str(&format!(
                    "  n{} -> n{};\n",
                    from, to));
                }
            });
    }

    s.push_str("}\n");
    s
}

fn escape_dot(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {

    #[test]
    fn dot_contains_nodes_and_edges() {
    }

    #[test]
    fn dot_colors_bool_true_as_red() {
    }
}

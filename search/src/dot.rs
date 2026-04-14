use crate::dag::MultiDag;

/// Options d'export DOT.
#[derive(Clone, Copy, Debug, Default)]
pub struct DotOptions {
    pub show_node_labels: bool,
    pub show_edge_multiplicity: bool,
}

/// Exporte un `MultiDag` au format Graphviz DOT.
pub fn to_dot(dag: &MultiDag, options: DotOptions) -> String {
    let mut s = String::new();
    s.push_str("digraph G {\n");
    s.push_str("  rankdir=TB;\n");
    s.push_str("  node [shape=box];\n");

    for node in &dag.nodes {
        if options.show_node_labels {
            // On utilise la forme canonique (ou l'arbre si présent) comme label.
            let label = match &node.tree {
                Some(t) => t.to_string(),
                None => node.canon.clone(),
            };
            s.push_str(&format!("  n{} [label=\"{}\"];\n", node.id, escape_dot(&label)));
        } else {
            s.push_str(&format!("  n{} [label=\"{}\"];\n", node.id, node.id));
        }
    }

    let mut edges: Vec<_> = dag.edges.iter().collect();
    edges.sort_by_key(|((from, to), _)| (*from, *to));
    for ((from, to), mult) in edges {
        if options.show_edge_multiplicity && *mult != 1 {
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
    use crate::tree::Tree;

    #[test]
    fn dot_contains_nodes_and_edges() {
        let t: Tree = "(()())".parse().unwrap();
        let dag = MultiDag::from_tree(&t, true);
        let dot = to_dot(
            &dag,
            DotOptions {
                show_node_labels: true,
                show_edge_multiplicity: true,
            },
        );
        assert!(dot.contains("digraph G"));
        assert!(dot.contains("->"));
        assert!(dot.contains("label=\"2\""));
    }
}

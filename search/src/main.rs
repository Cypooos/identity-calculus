use std::path::PathBuf;

use clap::{Parser, Subcommand};

use search::dag::MultiDag;
use search::dot::to_dot;
use search::motifs::find_diamond3;
use search::tree::Tree;

#[derive(Parser, Debug)]
#[command(name = "search")]
#[command(about = "Étude de DAGs de réduction d'arbres ordonnés enracinés", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Construit le DAG de réduction et affiche un résumé.
    Dag {
        /// Arbre en syntaxe canonique, ex: (()()).
        #[arg(long)]
        tree: String,

        /// Conserve/affiche les labels d'arbres (utile pour DOT).
        #[arg(long)]
        labels: bool,

        /// Écrit un export DOT dans un fichier.
        #[arg(long)]
        dot: Option<PathBuf>,

        /// Affiche les multiplicités sur les arêtes DOT (si > 1).
        #[arg(long)]
        dot_multiplicity: bool,
    },

    /// Énumère tous les arbres ayant N arêtes.
    Enumerate {
        #[arg(long)]
        edges: usize,
    },

    /// Cherche un motif "diamond3" sur des arbres jusqu'à N arêtes.
    SearchDiamond3 {
        #[arg(long)]
        max_edges: usize,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Dag {
            tree,
            labels,
            dot,
            dot_multiplicity,
        } => {
            let tree: Tree = tree.parse()?;
            let dag = MultiDag::from_tree(&tree, labels);

            println!("nodes: {}", dag.node_count());
            println!("distinct_edges: {}", dag.distinct_edge_count());
            println!("total_multiplicity: {}", dag.total_multiplicity());
            println!("level_sizes: {:?}", dag.level_sizes());

            if let Some(path) = dot {
                let dot_str = to_dot(&dag, dot_multiplicity);
                std::fs::write(path, dot_str)?;
            }
        }
        Command::Enumerate { edges } => {
            let trees = Tree::enumerate_canonical_by_edges(edges);
            eprintln!("count: {}", trees.len());
            for t in trees {
                println!("{t}");
            }
        }
        Command::SearchDiamond3 { max_edges } => {
            for edges in 0..=max_edges {
                let trees = Tree::enumerate_canonical_by_edges(edges);
                for t in trees {
                    let tree: Tree = t.parse()?;
                    if let Some(w) = find_diamond3(&tree) {
                        println!("found: true");
                        println!("top: {}", w.top);
                        println!("middle: {:?}", w.middle);
                        println!("bottom: {}", w.bottom);
                        return Ok(());
                    }
                }
            }
            println!("found: false");
        }
    }

    Ok(())
}

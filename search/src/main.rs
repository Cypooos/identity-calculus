use std::path::PathBuf;

use clap::{Parser, Subcommand};

use itertools::Itertools;
use search::dag::MultiDag;
use search::dot::to_dot;
use search::enumerate::{catalan, get_hook_length, get_nb_reductions, get_trees};
use search::parse::parse_tree;
use search::tree::Tree;

use search::reconstruct::{get_spine, reconstruct};

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
        multi: bool,

        /// Color the spine
        #[arg(long)]
        spine: bool,

        /// Run the algorithm
        #[arg(long)]
        algo: bool,
    },

    /// Énumère le nombre de réduction d'un arbre. Si on donne un nombre, fait tout les arbres à n arretes
    Enumerate {
        #[arg(long)]
        tree: String,
        #[arg(long)]
        multi: bool,
    },

    /// Teste si deux dag sont égaux.
    Equality {
        #[arg(long)]
        tree1: String,
        #[arg(long)]
        tree2: String,
        #[arg(long)]
        multi: bool,
    },

    /// Currently, search for all pairs if trees of nodes <= nb_nodes if dag equality <=> canonical 
    SearchEquality {
        #[arg(long)]
        n:usize,
        #[arg(long)]
        multi: bool,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Dag {
            tree,
            labels:_,
            dot,
            multi,
            spine,
            algo,
        } => {
            let tree: Tree = tree.parse()?;
            let dag = MultiDag::<Tree>::from_tree(&tree);

            println!("nodes: {}", dag.node_count());
            println!("distinct_edges: {}", dag.edges_count_single());
            println!("total_multiplicity: {}", dag.edges_count());
            println!("finals: {:?}", dag.get_finals().iter().map(|x|format!("{}",dag.nodes.get(*x).unwrap().label)).collect::<Vec<String>>());
            
            if let Some(path) = dot {
                let dot_str = if algo {
                    to_dot(&reconstruct(&dag), multi)
                } else if spine {
                    let spine = get_spine(&dag);
                    to_dot(&dag.tag_vec(&spine), multi)
                } else {
                    to_dot(&dag,multi)
                };
                std::fs::write(path, dot_str)?;
            }
        }
        Command::Enumerate { tree, multi } => {
            match parse_tree(&tree) {
                Err(x) => println!("{x}"),
                Ok(tree) => {
                    let width = 2*tree.edge_count()+2;
                    let hook = get_hook_length(&tree);
                    println!("Table for {tree} with {} edges.",tree.edge_count());
                    let tree_to_nb = get_nb_reductions(tree, multi);
                    let v = tree_to_nb.iter().sorted_by(|x,y| Ord::cmp(&x.0.edge_count(),&y.0.edge_count()));
                    for (k,v) in v {
                        println!("[{0:>3}] tree {1:>width$}: {v:>5} reductions. ({hook:?})", k.edge_count(), format!("{}",k));
                    }
                    
                }
            }
        }

        Command::Equality { tree1, tree2, multi  } => {
            match (parse_tree(&tree1), parse_tree(&tree2)) {
                (Err(x),_) | (_, Err(x)) => println!("{x}"),
                (Ok(tree1), Ok(tree2)) => {
                    let mut dag1 = MultiDag::<Tree>::from_tree(&tree1);
                    let mut dag2 = MultiDag::<Tree>::from_tree(&tree2);
                    if multi {
                        println!("{tree1} is {}equal to {tree2}", if dag1 == dag2 {""} else {"not "});
                    } else {
                        dag1.remove_multiedges();
                        dag2.remove_multiedges();
                        println!("{tree1} is {}equal to {tree2}", if dag1 == dag2 {""} else {"not "});
                    }
                    
                }
            }
        }

        Command::SearchEquality{n,multi} => {
            let nb = catalan(n+1)*catalan(n+1);
            let mut i = 0;
            get_trees(n+1).iter_mut().for_each(|t1| {
                get_trees(n+1).iter_mut().for_each(|t2| {
                    i+=1;
                    if i % 10000 == 0 {println!("Done {i}/{nb}.")};
                    if t1 < t2 {return;}
                    let mut d1 = MultiDag::from_tree(&t1);
                    let mut d2 = MultiDag::from_tree(&t2);
                    if multi {
                        d1.remove_multiedges();
                        d2.remove_multiedges();
                    }
                    let s1 = format!("{t1}");
                    let s2 = format!("{t1}");
                    t1.to_canonical();
                    t2.to_canonical();
                    if (d1==d2) != (t1==t2) {
                        println!("[ALERT] for {s1} & {s2}: {} and {} with cano={t1}",d1==d2,t1==t2);
                    }
                });
            });
            println!("DONE ({i}/{nb}).")
        }
    }

    Ok(())
}

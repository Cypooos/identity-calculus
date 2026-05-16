//! Réduction par suppression de feuilles dans des arbres ordonnés enracinés,
//! et construction du DAG de réduction (avec multi-arêtes).

pub mod dag;
pub mod dot;
// pub mod motifs;
pub mod parse;
pub mod reduction;
pub mod tree;
pub mod reconstruct;
pub mod enumerate;
pub mod search;

pub use dag::{MultiDag, NodeId};
pub use parse::TreeParseError;
pub use reduction::immediate_reductions;
pub use tree::Tree;

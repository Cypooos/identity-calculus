use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Mutex, OnceLock};

use crate::reduction::immediate_reductions;
use crate::tree::Tree;

use itertools::Itertools;

pub type NodeId = usize;

#[derive(Clone, Debug)]
pub struct DagNode<T> {
    pub id: NodeId,
    pub label: T,
}

#[derive(Clone, Debug)]
pub struct MultiDag<T = Tree> {
    pub nodes: Vec<DagNode<T>>,
    pub edges: std::collections::HashMap<NodeId,Vec<NodeId>>, // with multiplicity or not, from parent -> child
    pub rev_edges: std::collections::HashMap<NodeId,Vec<NodeId>>, // reverse edges, from child -> parent. To be computed once
    pub root: NodeId, // the overall parent
}

impl<T> Default for MultiDag<T> {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            edges: std::collections::HashMap::new(),
            rev_edges: std::collections::HashMap::new(),
            root: 0,
        }
    }
}

impl<T> DagNode<T> {
    pub fn new(id:NodeId, label:T) -> Self{
        DagNode { id, label }
    }
}

impl<T> MultiDag<T> {

    fn get_new_id(&self) -> NodeId { self.nodes.len() }

    /// Add an node, with correct asserts.
    pub fn add_node(&mut self, label:T, parents:Vec<NodeId>, childs:Vec<NodeId>) -> NodeId {
        assert!(parents.iter().all(|x|*x < self.nodes.len()));
        let id: usize = self.get_new_id();
        self.nodes.push(DagNode::new(id,label));
        parents.iter().for_each(|x|self.edges.get_mut(x).unwrap().push(id));
        self.rev_edges.insert(id, parents);
        childs.iter().for_each(|x|self.rev_edges.get_mut(x).unwrap().push(id));
        self.edges.insert(id, childs);
        id
    }

    /// Add an edge, with correct asserts.
    pub fn add_edge(&mut self, parent:NodeId, child:NodeId) {
        assert!(child < self.nodes.len());
        assert!(parent < self.nodes.len());
        self.edges.get_mut(&parent).unwrap().push(child);
        self.rev_edges.get_mut(&child).unwrap().push(parent);
    }

    /// Apply `f : (label, NodeId) -> label` to all nodes
    pub fn map<U>(self, mut f: impl FnMut(T,NodeId) -> U) -> MultiDag<U> {
        MultiDag {
            nodes: self
                .nodes
                .into_iter()
                .map(|n| DagNode {
                    id: n.id,
                    label: f(n.label,n.id),
                })
                .collect(),
            edges: self.edges,
            rev_edges: self.rev_edges,
            root: self.root,
        }
    }

    /// Remove labels of the Dag
    pub fn remove_labels(self) -> MultiDag<()> {
        self.map(|_,_|())
    }

    /// Tag all nodes in set with true
    pub fn tag_set(self,set:&HashSet<NodeId>) -> MultiDag<(T,bool)> {
        self.map(|lbl,n|(lbl,set.contains(&n)))
    }

    /// Tag all nodes in vec with true
    pub fn tag_vec(self,vec:&Vec<NodeId>) -> MultiDag<(T,bool)> {
        self.map(|lbl,n|(lbl,vec.contains(&n)))
    }

    /// Remove multi-edges
    pub fn remove_multiedges(&mut self) {
        self.edges
            .iter_mut()
            .for_each(|(_,v)| *v = v.iter().unique().map(|x|*x).collect::<Vec<NodeId>>());
    }

    /// Number of vertices
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Number of edges (sum of multiplicities)
    pub fn edges_count(&self) -> usize {
        self.edges.iter().fold(0, |acc,(_,v)| acc+ v.len())
    }

    /// Number of edges (no multiplicities)
    pub fn edges_count_single(&self) -> usize {
        self.edges.iter().fold(0, |acc,(_,v)| acc+ v.iter().unique().collect::<Vec<&NodeId>>().len())
    }

    pub fn height(&self) -> usize {
        // TODO
        return 4;
    }

    /// Childs of a vertex, vec of (child, multiplicity)
    pub fn children(&self, from: NodeId) -> Vec<(NodeId, usize)> {
        self.edges
            .get(&from)
            .unwrap()
            .clone()
            .iter()
            .sorted()
            .chunk_by(|e|*e)
            .into_iter()
            .map(|(k,group)|(*k,group.count()))
            .collect()
    }

    /// Childs of a vertex, vec of (child, multiplicity)
    pub fn parents(&self, from: NodeId) -> Vec<(NodeId, usize)> {
        self.rev_edges
            .get(&from)
            .unwrap()
            .clone()
            .iter()
            .sorted()
            .chunk_by(|e|*e)
            .into_iter()
            .map(|(k,group)|(*k,group.count()))
            .collect()
    }

    /// Get all nodes stricly above a specific node
    pub fn get_above(&self, start:NodeId) -> HashSet<NodeId> {
        let mut to_tag = HashSet::new();
        let mut to_do = VecDeque::new();
        to_do.push_back(start);

        while let Some(e) = to_do.pop_front() {
            let parents = self.parents(e);
            for (node,_) in parents {
                if to_tag.insert(node) {
                    to_do.push_back(node);
                }
            }
        }
        return to_tag;
    }

    /// Get all nodes stricly below a specific node
    pub fn get_below(&self, start:NodeId) -> HashSet<NodeId> {
        let mut to_tag = HashSet::new();
        let mut to_do = VecDeque::new();
        to_do.push_back(start);

        while let Some(e) = to_do.pop_front() {
            let childs = self.children(e);
            for (node,_) in childs {
                if to_tag.insert(node) {
                    to_do.push_back(node);
                }
            }
        }
        return to_tag;
    }

    /// Tag with `true` all nodes stricly above a specific node
    pub fn tag_above(self, start:NodeId) -> MultiDag<(T,bool)> {
        let above = self.get_above(start);
        self.map(|x,n|(x,above.contains(&n)))
    }

    /// Tag with `true` all nodes stricly below a specific node
    pub fn tag_below(self, start:NodeId) -> MultiDag<(T,bool)> {
        let below = self.get_below(start);
        self.map(|x,n|(x,below.contains(&n)))
    }

    /// Returns the list of final elements of the dag (outdegree = 0)
    pub fn get_finals(&self) -> Vec<NodeId> {
        self.nodes.iter().map(|x|x.id).filter(|x|self.children(*x).is_empty()).collect()
    }

    pub fn get_final_nodes(&self) -> Vec<&DagNode<T>> {
        self.get_finals().iter().map(|x|self.nodes.get(*x).unwrap()).collect()
    }
}


impl MultiDag<Tree> {
    /// Construit le DAG de réduction (avec multi-arêtes) depuis un arbre racine.
    ///
    /// Les sommets sont fusionnés par forme canonique (affichage parenthésé sans espaces).
    pub fn from_tree(root: &Tree) -> Self {

        let mut dag = MultiDag {
            ..Default::default()
        };

        let root_id = dag.add_node(root.clone(),vec![], vec![]);
        dag.root = root_id;

        let mut queue = VecDeque::new();
        let mut done = HashSet::<NodeId>::new();
        let mut map = HashMap::<Tree,NodeId>::new();
        queue.push_back((root_id,root.clone()));

        while let Some((parent_id,parent_tree)) = queue.pop_front() {
            if !done.insert(parent_id) {
                continue;
            }
            for child in immediate_reductions(&parent_tree) {
                if let Some(child_id) = map.get(&child) {
                    dag.add_edge(parent_id, *child_id);
                } else {
                    let child_id = dag.add_node(child.clone(), vec![parent_id],vec![]);
                    map.insert(child.clone(), child_id);
                    queue.push_back((child_id,child));
                }
            }
        }
        dag
    }
}

static EQ_MEMO: OnceLock<Mutex<HashMap<(Tree, Tree), bool>>> = OnceLock::new();

fn eq_memo() -> &'static Mutex<HashMap<(Tree, Tree), bool>> {
    EQ_MEMO.get_or_init(|| Mutex::new(HashMap::new()))
}

impl PartialEq for MultiDag<Tree> {
    fn eq(&self, other: &Self) -> bool {
        fn check_rec(
            me: &MultiDag<Tree>,
            node1: NodeId,
            other: &MultiDag<Tree>,
            node2: NodeId,
        ) -> bool {
            let left = me.nodes.get(node1).unwrap();
            let right = other.nodes.get(node2).unwrap();

            let key = (left.label.clone(), right.label.clone());

            if let Some(&res) = eq_memo().lock().unwrap().get(&key) {
                return res;
            }

            let res = if left.label == right.label {
                true
            } else {
                
                let left_children: Vec<(usize, Vec<NodeId>)> =
                    me.children(node1)
                        .into_iter()
                        .sorted_by_key(|&(_, ar)| ar)
                        .chunk_by(|&(_, ar)| ar)
                        .into_iter()
                        .map(|(ar, group)| {
                            let nodes = group.map(|(node, _)| node).collect();
                            (ar, nodes)
                        })
                        .collect();

                let right_children: Vec<(usize, Vec<NodeId>)> =
                    other.children(node2)
                        .into_iter()
                        .sorted_by_key(|&(_, ar)| ar)
                        .chunk_by(|&(_, ar)| ar)
                        .into_iter()
                        .map(|(ar, group)| {
                            let nodes = group.map(|(node, _)| node).collect();
                            (ar, nodes)
                        })
                        .collect();

                        
                if left_children.len() != right_children.len() || left_children.iter().zip(&right_children).any(|((ar1,v1),(ar2,v2))| *ar1 != *ar2 || v1.len() != v2.len()) {
                    false
                } else {
                    left_children
                        .iter()
                        .zip(right_children)
                        .all(|((_,left),(_,right))|
                            left
                            .iter()
                            .permutations(left.len())
                            .any(|perm| {
                                right
                                    .iter()
                                    .zip(perm)
                                    .all(|(x, y)| check_rec(me, *x, other, *y))
                            })
                        )
                }
            };
            eq_memo().lock().unwrap().insert(key, res);
            res
        }

        check_rec(self, self.root, other, other.root)
    }
}


// impl MultiDag<()> {
//     pub fn from_tree(root:&Tree) -> Self {
//         MultiDag::<Tree>::from_tree(root).remove_labels()
//     }
// }



#[cfg(test)]
mod test {
    use crate::{MultiDag, parse::parse_tree};


    #[test]
    fn eq_1() {
        let d1 = MultiDag::from_tree(&parse_tree("(()(()))").unwrap());
        let d2 = MultiDag::from_tree(&parse_tree("((())())").unwrap());
        assert_eq!(d1,d2);
    }


    #[test]
    fn eq_2() {
        let d1 = MultiDag::from_tree(&parse_tree("( (()()) (((())())) )").unwrap());
        let d2 = MultiDag::from_tree(&parse_tree("( ((()(()))) (()()) )").unwrap());
        assert_eq!(d1,d2);
    }

    #[test]
    fn neq_1() {
        let d1 = MultiDag::from_tree(&parse_tree("((())(()))").unwrap());
        let d2 = MultiDag::from_tree(&parse_tree("((())())").unwrap());
        assert_ne!(d1,d2);
    }
    
    #[test]
    fn counter_example() {
        let d1 = MultiDag::from_tree(&parse_tree("( (()(())) (()(())) )").unwrap());
        let d2 = MultiDag::from_tree(&parse_tree("( (()(())) ((())()) )").unwrap());


        assert_ne!(d1,d2);
    }
    
    #[test]
    fn gpt_example() {
        let d1 = MultiDag::from_tree(&parse_tree("(   ( ( ()(()) ) () )  ( (()(())) () )   )").unwrap());
        let d2 = MultiDag::from_tree(&parse_tree("(   ( ( ()(()) ) () )  ( ((())()) () )   )").unwrap());

        assert_ne!(d1,d2);
    }
}

/*

I
\x.x

N
( (I ( (I I) I))  (I I))

M
( ((I I) (I I))  (I I))

*/

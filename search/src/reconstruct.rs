use std::collections::VecDeque;


use crate::{dag::*, dot::DotLabel};
// use crate::tree::Tree;

pub enum NodeType {
    Spine1, Spine2, Both, None
}

impl DotLabel for NodeType {
    fn get_meta(&self) -> String {
        match self {
            Self::Spine1 => format!("color=red"),
            Self::Spine2 => format!("color=green"),
            Self::Both => format!("color=yellow"),
            Self::None => format!(""),
        }
    }
}

/// Get the spine of the dag, in order from final to toppest
pub fn get_spine<T>(m:&MultiDag<T>) -> Vec<NodeId> {
    let mut to_tag = Vec::new();
    let mut to_do = VecDeque::new();

    let finals = m.get_finals();
    let root = if finals.len() != 1 {
        panic!("More than a single final");
    } else { finals.get(0).unwrap()};

    to_do.push_back(*root);
    to_tag.push(*root);
    while let Some(e) = to_do.pop_front() {
        let parents = m.parents(e);
        for (node,_) in parents {
            let childs_of_parent = m.children(node);
            if childs_of_parent.len() == 1 && let Some((me,1)) = childs_of_parent.get(0) {
                assert!(*me == e);
                if !to_tag.contains(&node) {
                    to_tag.push(node);
                    to_do.push_back(node);
                }
            }
        }
    }
    to_tag
}

/// Get the spine of the dag ignoring marked vertices, in order from final to toppest
pub fn get_spine_ignorable<T>(m:&MultiDag<(T,bool)>, start:NodeId) -> Vec<NodeId> {
    let mut to_tag = Vec::new();
    let mut to_do = VecDeque::new();

    to_do.push_back(start);
    to_tag.push(start);
    while let Some(e) = to_do.pop_front() {
        let parents = m.parents(e);
        for (node,_) in parents {
            if m.nodes.get(node).unwrap().label.1 { continue;};
            let childs_of_parent = m.children(node).into_iter().filter(|(node,_)|!m.nodes.get(*node).unwrap().label.1 ).collect::<Vec<(NodeId,usize)>>();
            if childs_of_parent.len() == 1 && let Some((me,1)) = childs_of_parent.get(0) {
                assert!(*me == e);
                if !to_tag.contains(&node) {
                    to_tag.push(node);
                    to_do.push_back(node);
                }
            }
        }
    }
    to_tag
}

/// find the first parent not in the spine, and the spine position 
pub fn find_spine_split<T>(m:&MultiDag<T>, spine:Vec<NodeId>) -> Option<(NodeId,usize)> {
    for (i,node) in spine.iter().enumerate() {
        let parents = m.rev_edges.get(node).unwrap();
        if parents.len() > 1 {
            let good_parent = parents.iter().find(|x|!spine.contains(x));
            return good_parent.map(|x|(*x,i));
        }
    }
    return None;
}

/// get all spine splits
pub fn all_spine_split<T>(m:&MultiDag<T>, spine:&Vec<NodeId>) -> Vec<(NodeId,usize)> {
    let mut nb = 2;
    let mut res = Vec::new();
    for (i,node) in spine.iter().enumerate() {
        let parents = m.rev_edges.get(node).unwrap();
        if parents.len() > nb {
            if let Some(good_parent) = parents.iter().find(|x|!spine.contains(x)) {
                res.push((*good_parent,i));
            }
            nb+=1;
        }
    }
    return res
}


pub fn reconstruct<T : Clone>(m:&MultiDag<T>) -> MultiDag<(T,bool)> {
    
    let spine = get_spine(&m);
    let splits = all_spine_split(m,&spine); // correct
    let (_,first_split_pos) = *splits.get(0).unwrap();
    let spine_above_first_split = *spine.get(first_split_pos+1).unwrap();
    let mut above_first_split = m.get_above(spine_above_first_split);
    above_first_split.insert(spine_above_first_split);

    return m.clone().tag_set(&above_first_split);



    // if let Some((node_bifurcation,i)) = find_spine_split(m, spine) {
    //     //println!("{}",m.nodes.);
    //     let spine2 = get_spine_ignorable(&tagged_multi,node_bifurcation);
    //     let tagged_multi2 = tagged_multi.tag_vec(&spine2);
    //     // node = spine[i] contains a bifurcation
        
    // }

    // panic!("spine ez")
}
use fnv::FnvHashMap;

use gfa2::{
    gfa2::{
        Edge, 
        Segment, 
        GroupO, 
        GroupU, 
        GFA2, 
        orientation::Orientation,
    },
    tag::OptFields,
};

use crate::{
    handle::{Edge as GraphEdge, Handle, NodeId},
    handlegraph::iter::*,
    handlegraph::HandleGraph,
    mutablehandlegraph::MutableHandleGraph,
    pathgraph::PathHandleGraph,
};

use super::{Node, Path, PathId};

#[derive(Debug)]
pub struct HashGraph {
    pub max_id: NodeId,
    pub min_id: NodeId,
    pub graph: FnvHashMap<NodeId, Node>,
    pub path_id: FnvHashMap<Vec<u8>, i64>,
    pub paths: FnvHashMap<i64, Path>,
}

impl Default for HashGraph {
    fn default() -> HashGraph {
        HashGraph {
            max_id: NodeId::from(0),
            min_id: NodeId::from(std::u64::MAX),
            graph: Default::default(),
            path_id: Default::default(),
            paths: Default::default(),
        }
    }
}

impl HashGraph {
    pub fn new() -> HashGraph {
        Default::default()
    }

    fn add_gfa_segment<'a, 'b, T: OptFields>(
        &'a mut self,
        seg: &'b Segment<usize, T>,
    ) {
        self.create_handle(&seg.sequence, seg.id as u64);
    }

    fn add_gfa_link<T: OptFields>(&mut self, link: &Edge<usize, T>) {
        let left_len = link.sid1.to_string().len();
        let right_len = link.sid2.to_string().len();

        let left_orient = match &link.sid1.to_string()[left_len-1..] {
            "0" => Orientation::Forward,
            "1" => Orientation::Backward,
            _ => panic!("Error! Edge did not include orientation"),
        };
        let right_orient = match &link.sid2.to_string()[right_len-1..] {
            "0" => Orientation::Forward,
            "1" => Orientation::Backward,
            _ => panic!("Error! Edge did not include orientation"),
        };

        let left_id = &link.sid1.to_string()[..left_len-1];
        let right_id = &link.sid2.to_string()[..right_len-1];
        
        let left = Handle::new(left_id.parse::<u64>().unwrap() as u64, left_orient);
        let right = Handle::new(right_id.parse::<u64>().unwrap() as u64, right_orient);

        self.create_edge(GraphEdge(left, right));
    }

    fn add_gfa_path_o<T: OptFields>(&mut self, path: &GroupO<usize, T>) {
        let path_id = self.create_path_handle(&path.id, false);
        for (name, orient) in path.iter() {
            self.append_step(&path_id, Handle::new(name as u64, orient));
        }
    }

    // the U-Group encodes a subgraph and all the segments id that are 
    // presents in the var_field section do not have an orientation!
    // by default we should consider to have Forward (+) orientation? 
    fn add_gfa_path_u<T: OptFields>(&mut self, path: &GroupU<usize, T>) {
        let path_id = self.create_path_handle(&path.id, false);
        for name in path.iter() {
            self.append_step(&path_id, Handle::new(name as u64, Orientation::Forward));
        }
    }

    // TODO: add fragment and gap lines?
    pub fn from_gfa<T: OptFields>(gfa: &GFA2<usize, T>) -> HashGraph {
        let mut graph = Self::new();
        gfa.segments.iter().for_each(|s| graph.add_gfa_segment(s));
        gfa.edges.iter().for_each(|l| graph.add_gfa_link(l));
        gfa.groups_o.iter().for_each(|o| graph.add_gfa_path_o(o));
        gfa.groups_u.iter().for_each(|u| graph.add_gfa_path_u(u));
        graph
    }

    pub fn print_path(&self, path_id: &PathId) {
        let path = self.paths.get(&path_id).unwrap();
        println!("Path\t{}", path_id);
        for (ix, handle) in path.nodes.iter().enumerate() {
            let node = self.get_node(&handle.id()).unwrap();
            if ix != 0 {
                print!(" -> ");
            }
            print!("{}", node.sequence);
        }

        println!();
    }

    pub fn print_occurrences(&self) {
        self.all_handles().for_each(|h| {
            let node = self.get_node(&h.id()).unwrap();
            println!("{} - {:?}", node.sequence, node.occurrences);
        });
    }

    pub fn get_node(&self, node_id: &NodeId) -> Option<&Node> {
        self.graph.get(node_id)
    }

    pub fn get_node_unchecked(&self, node_id: &NodeId) -> &Node {
        self.graph.get(node_id).unwrap_or_else(|| {
            panic!("Tried getting a node that doesn't exist, ID: {:?}", node_id)
        })
    }

    pub fn get_node_mut(&mut self, node_id: &NodeId) -> Option<&mut Node> {
        self.graph.get_mut(node_id)
    }

    pub fn get_path(&self, path_id: &PathId) -> Option<&Path> {
        self.paths.get(path_id)
    }

    pub fn get_path_unchecked(&self, path_id: &PathId) -> &Path {
        self.paths
            .get(path_id)
            .unwrap_or_else(|| panic!("Tried to look up nonexistent path:"))
    }
}
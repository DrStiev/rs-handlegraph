use fnv::FnvHashMap;
use bstr::BString;

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

/// New type
/// # Example
/// ```ignore
/// pub struct HashGraph {
///     pub max_id: NodeId,
///     pub min_id: NodeId,
///     pub graph: FnvHashMap<NodeId, Node>,
///     pub path_id: FnvHashMap<Vec<u8>, i64>,
///     pub paths: FnvHashMap<i64, Path>,
/// }
/// ```
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

    // TODO: handle reverse and complement 
    fn add_gfa_link<T: OptFields>(&mut self, link: &Edge<usize, T>) {
        let left_len = link.sid1.to_string().len();
        let right_len = link.sid2.to_string().len();

        let left_id: String = link.sid1.to_string()[..left_len-1].to_string();
        let right_id: String = link.sid2.to_string()[..right_len-1].to_string();

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
        
        let left = Handle::new(left_id.parse::<u64>().unwrap() as u64, left_orient);
        let right = Handle::new(right_id.parse::<u64>().unwrap() as u64, right_orient);
        self.create_edge(GraphEdge(left, right));
        /*
        let id_left = self.get_node(&left.id()).unwrap().clone();
        let id_right = self.get_node(&right.id()).unwrap().clone();

        println!("left id: {}\nright id: {}", id_left.sequence, id_right.sequence);

        let mut node_ids: Vec<_> = self.graph.keys().collect();
        node_ids.sort();
        println!("Nodes & edges");
        for id in node_ids.iter() {
            let node = self.graph.get(id).unwrap();
            let seq: &BStr = node.sequence.as_ref();
            println!("  {:2}\t{}", u64::from(**id), seq);
            let lefts: Vec<_> =
                node.left_edges.iter().map(|x| u64::from(x.id())).collect();
            println!("  Left edges:  {:?}", lefts);
            let rights: Vec<_> =
                node.right_edges.iter().map(|x| u64::from(x.id())).collect();
            println!("  Right edges: {:?}", rights);
        }
        print!("\n");

        // the reverse and complement operation it's performed too many times
        // seems that the graph lose trace about the first edge after the reverse complement
        // operation
        let left_reversed: Handle = if left.is_reverse() {
            let id = self.get_node(&left.id()).unwrap().clone();
            let new_left = self.create_handle(&id.sequence, left.id()).clone();
            self.apply_orientation(new_left.flip()).clone()
        } else { left }; 
        let right_reversed: Handle = if right.is_reverse() {
            let id = self.get_node(&right.id()).unwrap().clone();
            let new_right = self.create_handle(&id.sequence, right.id()).clone();
            self.apply_orientation(new_right.flip()).clone()
        } else { right };

        //let right_reversed: Handle = right;
        //let left_reversed: Handle = left;

        //println!("left reversed: {}", self.get_node(&left_reversed.id()).unwrap().clone().sequence);
        //println!("right reversed: {}", self.get_node(&right_reversed.id()).unwrap().clone().sequence);

        let mut node_ids: Vec<_> = self.graph.keys().collect();
        node_ids.sort();
        println!("Nodes & edges");
        for id in node_ids.iter() {
            let node = self.graph.get(id).unwrap();
            let seq: &BStr = node.sequence.as_ref();
            println!("  {:2}\t{}", u64::from(**id), seq);
            let lefts: Vec<_> =
                node.left_edges.iter().map(|x| u64::from(x.id())).collect();
            println!("  Left edges:  {:?}", lefts);
            let rights: Vec<_> =
                node.right_edges.iter().map(|x| u64::from(x.id())).collect();
            println!("  Right edges: {:?}", rights);
        }
        print!("\n");

        self.create_edge(GraphEdge(left_reversed, right_reversed));
        */
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
    /// Function that takes a GFA2 object as input and return a HashGraph object
    /// # Example
    /// ```ignore
    /// use bstr::BStr;
    /// use gfa2::gfa2::GFA2;
    /// use gfa2::parser_gfa2::GFA2Parser;
    /// 
    /// let parser = GFA2Parser::new();
    /// let gfa: Option<GFA2<usize, ()>> = parser.parse_file("./tests/gfa2_files/spec_q7.gfa").ok();
    ///
    /// if let Some(gfa) = gfa {
    ///     let graph = HashGraph::from_gfa(&gfa);
    ///     println!("{:#?}", graph);
    /// } else {
    ///     panic!("Couldn't parse test GFA file!");
    /// }
    /// 
    /// /*
    /// HashGraph {
    /// max_id: NodeId(13),
    /// min_id: NodeId(11),
    ///     graph: {
    ///         NodeId(13): Node {
    ///             sequence: "CTTGATT",
    ///             left_edges: [
    ///                 Handle(24),
    ///                 Handle(23),
    ///             ],
    ///             right_edges: [],
    ///             occurrences: {0: 2},
    ///         },
    ///         NodeId(12): Node {
    ///             sequence: "TCAAGG",
    ///             left_edges: [
    ///                 Handle(26),
    ///             ],
    ///             right_edges: [
    ///                 Handle(23),
    ///             ],
    ///             occurrences: {0: 1},
    ///         },
    ///         NodeId(11): Node {
    ///             sequence: "ACCTT",
    ///             left_edges: [],
    ///             right_edges: [
    ///                 Handle(25),
    ///                 Handle(26),
    ///             ],
    ///             occurrences: {0: 0},
    ///         },
    ///     },
    ///     path_id: {
    ///         [49,52]: 0
    ///     },
    ///     paths: {
    ///         0: Path {
    ///             path_id: 0,
    ///             name: "14",
    ///             is_circular: false,
    ///             nodes: [
    ///                 Handle(22),
    ///                 Handle(25),
    ///                 Handle(26),
    ///             ],
    ///         },
    ///     },
    /// }
    /// */
    /// ```
    pub fn from_gfa<T: OptFields>(gfa: &GFA2<usize, T>) -> HashGraph {
        let mut graph = Self::new();
        gfa.segments.iter().for_each(|s| graph.add_gfa_segment(s));
        gfa.edges.iter().for_each(|l| graph.add_gfa_link(l));
        gfa.groups_o.iter().for_each(|o| graph.add_gfa_path_o(o));
        gfa.groups_u.iter().for_each(|u| graph.add_gfa_path_u(u));
        graph
    }

    /// Function that takes a HashGraph object as input and return a GFA2 object
    /// This function is still ```Work In Progress``` so it's not perfect.\
    /// Sometimes can leads to unexpected bugs.
    /// # Example
    /// ```ignore
    /// use bstr::BString;
    /// use gfa2::gfa2::GFA2;
    /// 
    /// let mut graph = path_graph();
    /// 
    /// // Add a path 3 -> 5
    /// let p1 = graph.create_path_handle(b"path-1", false);
    /// graph.append_step(&p1, H3);
    /// graph.append_step(&p1, H5);
    /// // Add another path 1 -> 3 -> 4 -> 6
    /// let p2 = graph.create_path_handle(b"path-2", false);
    /// graph.append_step(&p2, H1);
    /// let _p2_3 = graph.append_step(&p2, H3);
    /// let _p2_4 = graph.append_step(&p2, H4);
    /// graph.append_step(&p2, H6);
    /// let _test_node = |graph: &HashGraph,
    ///                  nid: u64,
    ///                  o1: Option<&usize>,
    ///                  o2: Option<&usize>| {
    ///     let n = graph.get_node(&NodeId::from(nid)).unwrap();
    ///     assert_eq!(o1, n.occurrences.get(&p1));
    ///     assert_eq!(o2, n.occurrences.get(&p2));
    /// };
    /// 
    /// let gfa2: GFA2<BString, ()> = HashGraph::to_gfa(graph);
    /// println!("{}", gfa2);
    ///  
    /// /*
    /// H       VN:Z:2.0
    /// S       1       42      A
    /// S       2       42      AA
    /// S       3       42      AAA
    /// S       4       42      AAAA
    /// S       5       42      AAAAA
    /// S       6       42      AAAAAA
    /// E       *       1+      2-      42      42$     42      42$     *
    /// E       *       1+      3-      42      42$     42      42$     *
    /// E       *       2+      1+      42      42$     42      42$     *
    /// E       *       2+      5-      42      42$     42      42$     *
    /// E       *       3+      1+      42      42$     42      42$     *
    /// E       *       3+      4-      42      42$     42      42$     *
    /// E       *       4+      3+      42      42$     42      42$     *
    /// E       *       4+      6-      42      42$     42      42$     *
    /// E       *       5+      2+      42      42$     42      42$     *
    /// E       *       5+      6-      42      42$     42      42$     *
    /// E       *       6+      5+      42      42$     42      42$     *
    /// E       *       6+      4+      42      42$     42      42$     *
    /// O       path-1   3+ 5+ 
    /// O       path-2   1+ 3+ 4+ 6+
    /// */
    /// ```
    pub fn to_gfa(graph: HashGraph) -> GFA2<BString, ()> {
        use gfa2::gfa2::*;
        
        // I think it can be more efficient but for now it's good 
        let mut file: GFA2<BString, ()> = GFA2::new();
        // default header
        file.headers = vec![
            Header::new(Some("VN:Z:2.0".into()))
        ];
        // TODO: for now this section it's not interesting so 
        // it can be ignored 
        file.fragments = vec![];
        file.gaps = vec![];
        file.segments = vec![];

        // TODO: the orientation is only retrieved partially 
        file.edges = vec![];
        file.groups_o = vec![];
        file.groups_u = vec![];

        let mut node_ids: Vec<_> = graph.graph.keys().collect();
        node_ids.sort(); 
        
        for nodeid in node_ids.iter() {
            let node = graph.graph.get(nodeid).unwrap();
            let seq_id = u64::from(**nodeid).to_string();
            
            // obtain all the segments part of the graph
            file.segments.push(
                Segment {
                    id: seq_id.clone().into(), 
                    // placeholder value
                    // the len value must be present but it's value it's not 
                    // as important as it's presence
                    len: "42".into(), 
                    sequence: node.sequence.clone(),
                    tag: (),
                });

            // obtain all the id associated to the left and right edges
            let lefts: Vec<_> = node.left_edges.iter().map(|x| u64::from(x.id())).collect();
            let rights: Vec<_> = node.right_edges.iter().map(|x| u64::from(x.id())).collect();
           // obtain all the orientation associated to the left and right edge ids
            let lefts_orient: Vec<_> = node.left_edges.iter().map(|x| bool::from(x.is_reverse())).collect();
            let right_orient: Vec<_> = node.right_edges.iter().map(|x| bool::from(x.is_reverse())).collect();

            let mut i: usize = 0;
            // iter over the left ids of the edge array
            for id in lefts { 
                let orient: String = if lefts_orient[i] {
                    "+".to_string()
                } else {
                    "-".to_string()
                };
                file.edges.push(
                    Edge {
                        // placeholder id
                        id: "*".into(),
                        // starting node
                        // placeholder orientation
                        sid1: format!("{}{}", seq_id.clone(), "+").into(), 
                        // placeholder orientation
                        // this would works fine if only the reverse and complement 
                        // in the gfa_add_link() function would work properly
                        sid2: format!("{}{}", id.to_string(), orient).into(), 
                        beg1:"42".into(), // placeholder value
                        end1:"42$".into(), // placeholder value
                        beg2:"42".into(), // placeholder value
                        end2:"42$".into(), // placeholder value
                        alignment: "*".into(),
                        tag: (),
                    }
                );
                i += 1;
            }

            i = 0;
            // iter over the right ids of the edge array
            for id in rights {
                let orient: String = if right_orient[i] {
                    "+".to_string()
                } else {
                    "-".to_string()
                };
                file.edges.push(
                    // push first edge left id then edge right id
                    Edge {
                        // placeholder id
                        id: "*".into(),
                       // starting node
                        // placeholder orientation
                        sid1: format!("{}{}", seq_id.clone(), "+").into(),
                        // placeholder orientation
                        // this would works fine if only the reverse and complement 
                        // in the gfa_add_link() function would work properly
                        sid2: format!("{}{}", id.to_string(), orient).into(), 
                        beg1:"42".into(), // placeholder value
                        end1:"42$".into(), // placeholder value
                        beg2:"42".into(), // placeholder value
                        end2:"42$".into(), // placeholder value
                        alignment: "*".into(),
                        tag: (),
                    }
                );
                i+= 1;
            }
        }

        let mut x :i64 = 0;
        // obtain all the information for the O-Group
        // TODO: this could incidentally cast the U-Group information as an O-Group 
        while !graph.get_path(&x).is_none() {
            let mut id_path: String = "".to_string();

            for nodeid in graph.get_path(&x).unwrap().nodes.iter() {
                let seq_id = u64::from(*nodeid).to_string();

                id_path = format!("{}{}{}{}", id_path, " ", seq_id, "+");
                
            }
            file.groups_o.push(
                GroupO::new(
                    graph.get_path(&x).unwrap().name.to_string().into(),
                    // get all the id and orientation for the path reference
                    id_path.clone().into(),
                    (),
                )
            );
            x += 1;
        }
        file
    }

    /// Function that print all the sequence associated to the segment ids 
    /// found in a certain path
    /// # Examples
    /// ```ignore
    /// use hashgraph::HashGraph::graph;
    /// use bstr::BStr;
    /// 
    /// let mut graph = HashGraph::new();
    /// let h1 = graph.create_handle(b"ACCTT", 11);
    /// let h2 = graph.create_handle(b"TCAAGG", 12);
    /// let h3 = graph.create_handle(b"CTTGATT", 13);
    /// 
    /// let p1 = graph.create_path_handle(b"path-1", false);
    /// graph.append_step(&p1, h1);
    /// graph.append_step(&p1, h2);
    /// graph.append_step(&p1, h3);
    /// 
    ///  let mut x :i64 = 0;
    /// while !graph.get_path(&x).is_none() {
    ///     // ACCTT -> TCAAGG -> CTTGATT
    ///     graph.print_path(&x);
    /// } 
    /// ```
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

    /// Function that returns a reference to the value corresponding to the key.\
    /// The reference is a Node object wrapped in Option
    /// # Examples
    /// ```ignore
    /// use hashgraph::HashGraph::graph;
    /// use bstr::BStr;
    /// 
    /// let mut graph = HashGraph::new();
    /// let h1 = graph.create_handle(b"ACCTT", 11);
    /// 
    /// // Some(Node { sequence: "ACCTT", left_edges: [], right_edges: [], occurrences: {} })
    /// println!("{:?}", graph.get_node(&11)); 
    /// ```
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

    /// Function that returns a reference to the value corresponding to the key.\
    /// The reference is a Path object wrapped in Option
    /// # Examples
    /// ```ignore
    /// use hashgraph::HashGraph::graph;
    /// use bstr::BStr;
    /// 
    /// let mut graph = HashGraph::new();
    /// let h1 = graph.create_handle(b"ACCTT", 11);
    /// let h2 = graph.create_handle(b"TCAAGG", 12);
    /// 
    /// let p1 = graph.create_path_handle(b"path-1", false);
    /// graph.append_step(&p1, h1);
    /// graph.append_step(&p1, h2);
    /// 
    /// // Some(Path { path_id: 0, name: "path-1", is_circular: false, nodes: [Handle(22), Handle(24)] })
    /// println!("{:?}", graph.get_node(&11)); 
    /// ```
    pub fn get_path(&self, path_id: &PathId) -> Option<&Path> {
        self.paths.get(path_id)
    }

    pub fn get_path_unchecked(&self, path_id: &PathId) -> &Path {
        self.paths
            .get(path_id)
            .unwrap_or_else(|| panic!("Tried to look up nonexistent path:"))
    }
}
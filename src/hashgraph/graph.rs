use fnv::FnvHashMap;

use gfa2::{
    gfa1::{Link, Segment as Segment1, GFA},
    gfa2::{orientation::Orientation, Edge, GroupO, GroupU, Segment, GFA2},
    tag::OptFields,
};

use crate::{
    handle::{Edge as GraphEdge, Handle, NodeId},
    handlegraph::*,
    mutablehandlegraph::*,
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
#[derive(Clone, Debug)]
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

    fn add_gfa2_segment<'a, 'b, T: OptFields>(&'a mut self, seg: &'b Segment<usize, T>) {
        self.create_handle(&seg.sequence, seg.id as u64);
    }

    fn add_gfa_edge<T: OptFields>(&mut self, link: &Edge<usize, T>) {
        let left_len = link.sid1.to_string().len();
        let right_len = link.sid2.to_string().len();

        let left_id: String = link.sid1.to_string()[..left_len - 1].to_string();
        let right_id: String = link.sid2.to_string()[..right_len - 1].to_string();

        let left_orient = match &link.sid1.to_string()[left_len - 1..] {
            "0" => Orientation::Forward,
            "1" => Orientation::Backward,
            _ => panic!("Error! Edge did not include orientation"),
        };
        let right_orient = match &link.sid2.to_string()[right_len - 1..] {
            "0" => Orientation::Forward,
            "1" => Orientation::Backward,
            _ => panic!("Error! Edge did not include orientation"),
        };

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
    ///     let graph = HashGraph::from_gfa2(&gfa);
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
    pub fn from_gfa2<T: OptFields>(gfa: &GFA2<usize, T>) -> HashGraph {
        // use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};

        /*
        // Provide a custom bar style
        let pb_seg = ProgressBar::new(1000);
        pb_seg.set_style(ProgressStyle::default_bar().template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] [{pos}/{pos}]",
        ));
        // Provide a custom bar style
        let pb_link = ProgressBar::new(1000);
        pb_link.set_style(ProgressStyle::default_bar().template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] [{pos}/{pos}]",
        ));
        // Provide a custom bar style
        let pb_patho = ProgressBar::new(1000);
        pb_patho.set_style(ProgressStyle::default_bar().template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] [{pos}/{pos}]",
        ));
        // Provide a custom bar style
        let pb_pathu = ProgressBar::new(1000);
        pb_pathu.set_style(ProgressStyle::default_bar().template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] [{pos}/{pos}]",
        ));
        */

        let mut graph = Self::new();
        //println!("Segments");
        gfa.segments
            .iter() /*.progress_with(pb_seg)*/
            .for_each(|s| graph.add_gfa2_segment(s));
        //println!("Edges");
        gfa.edges
            .iter() /*.progress_with(pb_link)*/
            .for_each(|l| graph.add_gfa_edge(l));
        //println!("O-Groups");
        gfa.groups_o
            .iter() /*.progress_with(pb_patho)*/
            .for_each(|o| graph.add_gfa_path_o(o));
        //println!("U-Groups");
        gfa.groups_u
            .iter() /*.progress_with(pb_pathu)*/
            .for_each(|u| graph.add_gfa_path_u(u));
        graph
    }

    fn add_gfa_segment<'a, 'b, T: OptFields>(&'a mut self, seg: &'b Segment1<usize, T>) {
        self.create_handle(&seg.sequence, seg.name as u64);
    }

    fn add_gfa_link<T: OptFields>(&mut self, link: &Link<usize, T>) {
        let left = Handle::new(link.from_segment as u64, link.from_orient);
        let right = Handle::new(link.to_segment as u64, link.to_orient);

        self.create_edge(GraphEdge(left, right));
    }

    fn add_gfa_path<T: OptFields>(&mut self, path: &gfa2::gfa1::Path<usize, T>) {
        let path_id = self.create_path_handle(&path.path_name, false);
        for (name, orient) in path.iter() {
            self.append_step(&path_id, Handle::new(name as u64, orient));
        }
    }

    /// Function that takes a GFA2 object as input and return a HashGraph object
    /// # Example
    /// ```ignore
    /// let parser = GFAParser::new();
    /// let gfa: Option<GFA<usize, ()>> = parser.parse_file("./tests/gfa2_files/spec_q7.gfa").ok();
    ///
    /// if let Some(gfa) = gfa {
    ///     let graph = HashGraph::from_gfa1(&gfa);
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
    pub fn from_gfa<T: OptFields>(gfa: &GFA<usize, T>) -> HashGraph {
        // use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};

        /*
        // Provide a custom bar style
        let pb_seg = ProgressBar::new(1000);
        pb_seg.set_style(ProgressStyle::default_bar().template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] [{pos}/{pos}]",
        ));
        // Provide a custom bar style
        let pb_link = ProgressBar::new(1000);
        pb_link.set_style(ProgressStyle::default_bar().template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] [{pos}/{pos}]",
        ));
        // Provide a custom bar style
        let pb_path = ProgressBar::new(1000);
        pb_path.set_style(ProgressStyle::default_bar().template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] [{pos}/{pos}]",
        ));
        */

        let mut graph = Self::new();
        //println!("Segments");
        gfa.segments
            .iter() /*.progress_with(pb_seg)*/
            .for_each(|s| graph.add_gfa_segment(s));
        //println!("Links");
        gfa.links
            .iter() /*.progress_with(pb_link)*/
            .for_each(|l| graph.add_gfa_link(l));
        //println!("Paths");
        gfa.paths
            .iter() /*.progress_with(pb_path)*/
            .for_each(|p| graph.add_gfa_path(p));
        graph
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
    /// let mut x :i64 = 0;
    /// while !graph.get_path(&x).is_none() {
    ///     // ACCTT -> TCAAGG -> CTTGATT
    ///     graph.print_path(&x);
    ///     x +=1;
    /// }
    /// ```
    pub fn print_path(&self, path_id: &PathId) {
        let path = self.paths.get(&path_id).unwrap();
        println!("Path\t{}", path_id);
        for (ix, handle) in path.nodes.iter().enumerate() {
            let node = self.get_node(&handle.id());
            if node.is_some() {
                if ix != 0 {
                    print!(" -> ");
                }
                print!("{}", node.unwrap().sequence);
            } else {
                print!(" -> This node do not exists anymore");
            }
        }
        println!();
    }

    /// Print an HashGraph object in a simplified way
    /// # Example
    /// ```ignore
    /// graph.print_simple_graph();
    /// /*
    /// Graph: {
    ///     Nodes: {
    ///         13: CTTGATT
    ///         12: TCAAGG
    ///         11: ACCTT
    ///     }
    ///     Edges: {
    ///         12- --> 13+
    ///         11+ --> 12-
    ///         11+ --> 13+
    ///     }
    ///     Paths: {
    ///         14: ACCTT -> CTTGATT
    ///         15: ACCTT -> CCTTGA -(TCAAGG) -> CTTGATT
    ///     }
    /// }
    /// */
    /// ```
    pub fn print_graph(&self) {
        println!("Graph: {{");
        // print all the segments
        self.print_segments();
        // print all the edges
        self.print_edges();
        // print all the paths
        self.print_paths();
        println!("}}");
    }

    /// Function that prints all the segments in a graph
    fn print_segments(&self) {
        use bstr::BString;

        println!("\tNodes: {{");
        // get all the nodeid and sequence associated with them
        for handle in self.all_handles() {
            let node_id: String = handle.id().to_string();
            let sequence: BString = self.sequence_iter(handle.forward()).collect();

            println!("\t\t{}: {}", node_id, sequence);
        }
        println!("\t}}");
    }

    /// Function that prints all the edges in a graph
    fn print_edges(&self) {
        println!("\tEdges: {{");
        // get all the link (edge) between nodes
        for edge in self.all_edges() {
            let GraphEdge(left, right) = edge;

            let from_node: String = if !left.id().to_string().is_empty() {
                left.id().to_string()
            } else {
                "NUL".to_string()
            };
            let to_node: String = if !right.id().to_string().is_empty() {
                right.id().to_string()
            } else {
                "NUL".to_string()
            };

            let mut left_orient: String = "".to_string();
            if from_node != "NUL" {
                if left.is_reverse() {
                    left_orient = "-".to_string();
                } else {
                    left_orient = "+".to_string();
                }
            }

            let mut right_orient: String = "".to_string();
            if to_node != "NUL" {
                if right.is_reverse() {
                    right_orient = "-".to_string();
                } else {
                    right_orient = "+".to_string();
                }
            }

            println!(
                "\t\t{}{} --> {}{}",
                from_node, left_orient, to_node, right_orient
            );
        }
        println!("\t}}");
    }

    /// Function that prints all the paths in a graph
    fn print_paths(&self) {
        use bio::alphabets::dna;
        use bstr::BString;

        println!("\tPaths: {{");
        // get all the path
        for path_id in self.paths_iter() {
            let path = self.paths.get(&path_id).unwrap();
            //get the id or path name of a path
            let name = &path.name;
            let mut first: bool = true;

            for (ix, handle) in path.nodes.iter().enumerate() {
                let node = self.get_node(&handle.id()).unwrap();
                if first {
                    first = false;
                    print!("\t\t{}: ", name);
                }
                if ix != 0 {
                    print!(" -> ");
                }
                // print correct reverse and complement sequence to display the correct path
                if handle.is_reverse() {
                    let rev_sequence: BString = dna::revcomp(node.sequence.as_slice()).into();
                    print!("{} -({})", rev_sequence, node.sequence);
                } else {
                    print!("{}", node.sequence);
                }
            }
            println!();
        }
        println!("\t}}");
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
        self.graph
            .get(node_id)
            .unwrap_or_else(|| panic!("Tried getting a node that doesn't exist, ID: {:?}", node_id))
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
    /// println!("{:?}", graph.get_path(&0));
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

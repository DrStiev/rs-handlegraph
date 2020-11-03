use crate::{
    handle::{Edge, Handle, NodeId},
    hashgraph::HashGraph,
    mutablehandlegraph::*,
    pathgraph::PathHandleGraph,
};

use gfa2::{
    gfa2::{
        Line, 
        GFA2, 
        Header, 
        Segment, 
        Edge as GFA2Edge, 
        GroupO, 
        orientation::Orientation
    },
    gfa1::{
        Link, 
        Segment as Segment1,
        Path, 
        GFA,
        Line as Line1,
    },
    tag::OptFields,
    parser_gfa2::GFA2Result,
    parser_gfa1::GFAResult,
};
use bstr::BString;

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
///     let graph = conversion::from_gfa2(&gfa);
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
pub fn from_gfa2<G, T>(gfa2: &GFA2<usize, T>) -> G
where
    G: Default + AdditiveHandleGraph + PathHandleGraph,
    T: OptFields,
{
    let mut graph: G = Default::default();

    for segment in gfa2.segments.iter() {
        assert!(segment.id.to_string().parse::<u64>().unwrap() > 0);
        let seq = &segment.sequence;
        graph.create_handle(seq, segment.id);
    }

    for link in gfa2.edges.iter() {
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

        graph.create_edge(Edge(left, right));
    }

    for path in gfa2.groups_o.iter() {
        let name = &path.id;
        let path_id = graph.create_path_handle(name, false);
        for (seg, orient) in path.iter() {
            let handle = Handle::new(seg, orient);
            graph.append_step(&path_id, handle);
        }
    }

    // the U-Group encodes a subgraph and all the segments id that are 
    // presents in the var_field section do not have orientation!
    // by default we should consider to have Forward (+) orientation? 
    for path in gfa2.groups_u.iter() {
        let name = &path.id;
        let path_id = graph.create_path_handle(name, false);
        for seg in path.iter() {
            let handle = Handle::new(seg, Orientation::Forward);
            graph.append_step(&path_id, handle);
        }
    }

    graph
}

pub fn fill_gfa2_lines<G, I, T>(graph: &mut G, gfa_lines: I) -> GFA2Result<()>
where
    G: AdditiveHandleGraph + PathHandleGraph,
    I: Iterator<Item = GFA2Result<Line<usize, T>>>,
    T: OptFields,
{
    for line in gfa_lines {
        let line = line?;
        match line {
            Line::Segment(v) => {
                let id = NodeId::from(v.id);
                graph.create_handle(&v.sequence, id);
            }
            Line::Edge(v) => {
                let left_len = v.sid1.to_string().len();
                let right_len = v.sid2.to_string().len();
                let left_orient = match &v.sid1.to_string()[left_len-1..] {
                    "0" => Orientation::Forward,
                    "1" => Orientation::Backward,
                    _ => panic!("Error! Edge did not include orientation"),
                };
                let right_orient = match &v.sid2.to_string()[right_len-1..] {
                    "0" => Orientation::Forward,
                    "1" => Orientation::Backward,
                    _ => panic!("Error! Edge did not include orientation"),
                };
                let left_id = &v.sid1.to_string()[..left_len-1];
                let right_id = &v.sid2.to_string()[..right_len-1];
                
                let left = Handle::new(left_id.parse::<u64>().unwrap() as u64, left_orient);
                let right = Handle::new(right_id.parse::<u64>().unwrap() as u64, right_orient);
                graph.create_edge(Edge(left, right));
            }
            Line::GroupO(v) => {
                let name = &v.id;
                let path_id = graph.create_path_handle(name, false);
                for (seg, orient) in v.iter() {
                    let handle = Handle::new(seg, orient);
                    graph.append_step(&path_id, handle);
                }
            }
            Line::GroupU(v) => {
                let name = &v.id;
                let path_id = graph.create_path_handle(name, false);
                for seg in v.iter() {
                    let handle = Handle::new(seg, Orientation::Forward);
                    graph.append_step(&path_id, handle);
                }
            }
            _ => (),
        }
    }

    Ok(())
}

/// Function that takes a HashGraph object as input and return a GFA2 object
/// This function is still ```Work In Progress``` so it's not perfect.\
/// Sometimes can leads to unexpected bugs.
/// # Example
/// ```ignore
/// use bstr::BString;
/// use gfa2::{
///     gfa2::GFA2,
///     parser_gfa2::GFA2Parser,
/// };
///
/// let parser = GFA2Parser::new();
/// let gfa_in: GFA2<usize, ()> = parser.parse_file("./tests/gfa2_files/spec_q7.gfa").unwrap();
///
/// let graph = HashGraph::from_gfa(&gfa_in);
/// let gfa_out: GFA2<BString, ()> = handlegraph2::conversion::to_gfa2(&graph);
///
/// println!("{}", gfa_out);
/// println!("{}", gfa_in);
///  
/// /* hashgraph to gfa2:
/// H   VN:Z:2.0
/// S   13  0   CTTGATT
/// S   12  0   TCAAGG
/// S   11  0   ACCTT
/// E   *   12- 13+ 0   0$  0   0$  0M
/// E   *   11+ 12- 0   0$  0   0$  0M
/// E   *   11+ 13+ 0   0$  0   0$  0M
/// O   14  11+ 12- 13+
/// */
/// 
/// /* original gfa2:
/// H	VN:Z:2.0
/// H	
/// S	11	5	ACCTT	
/// S	12	6	TCAAGG
/// S	13	7	CTTGATT
/// E	1	11+	12-	1	5$	2	6$	4M
/// E	1	12-	13+	0	5	0	5	5M
/// E	1	11+	13+	2	5$	0	3	3M
/// O	14	11+ 12- 13+
/// */
/// ```
pub fn to_gfa2(graph: &HashGraph) -> GFA2<BString, ()> {
    use crate::handlegraph::*;
    // I think it can be more efficient but for now it's good 
    let mut file = GFA2::new();

    // default header
    file.headers = vec![
        Header::new(Some("VN:Z:2.0".into()))
    ];
    
    for handle in graph.all_handles() {
        let seq_id = BString::from(handle.id().to_string());
        let sequence: BString = graph.sequence_iter(handle.forward()).collect();
        let len: BString = BString::from(sequence.len().to_string());

        let segment = Segment {
            id: seq_id, 
            len: len, 
            sequence: sequence,
            tag: (),
        };
        file.segments.push(segment);
    }

    let orient = |rev: bool| {
        if rev {
            "-"
        } else {
            "+"
        }
    };

    for edge in graph.all_edges() {
        let Edge(left, right) = edge;

        let sid1_id: String = left.id().to_string();
        let sid1_orient = orient(left.is_reverse());
        let sid1: BString = BString::from(format!("{}{}", sid1_id, sid1_orient));

        let sid2_id: String = right.id().to_string();
        let sid2_orient = orient(right.is_reverse());
        let sid2: BString = BString::from(format!("{}{}", sid2_id, sid2_orient));

        let edge = GFA2Edge {
            // placeholder id
            id: "*".into(),
            sid1: sid1, 
            sid2: sid2, 
            beg1:"0".into(), // placeholder value
            end1:"0$".into(), // placeholder value
            beg2:"0".into(), // placeholder value
            end2:"0$".into(), // placeholder value
            alignment: "0M".into(),
            tag: (),
        };
        file.edges.push(edge);
    }

    for path_id in graph.paths_iter() {
        let path_name: BString = graph.path_handle_to_name(path_id).into();
        let mut segment_names: Vec<String>= Vec::new();

        for step in graph.steps_iter(path_id) {
            let handle = graph.handle_of_step(&step).unwrap();
            let segment: String = handle.id().to_string();
            let orientation = orient(handle.is_reverse());

            segment_names.push(segment);
            segment_names.push(orientation.to_string());
            segment_names.push(" ".to_string());
        }

        let segment_names: String = 
            segment_names.iter().fold(String::new(), |acc, str| acc + &str.to_string());

        let ogroup: GroupO<BString, _> = 
            GroupO::new(path_name, BString::from(segment_names), ());
        file.groups_o.push(ogroup);
    }

    file
}

/// Function that takes a GFA object as input and return a HashGraph object
/// # Example
/// ```ignore
/// let parser = GFAParser::new();
/// let gfa: Option<GFA<usize, ()>> = parser.parse_file("./tests/gfa2_files/file.gfa").ok();
///
/// if let Some(gfa) = gfa {
///     let graph = conversion::from_gfa(&gfa);
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
pub fn from_gfa<G, T>(gfa: &GFA<usize, T>) -> G
where
    G: Default + AdditiveHandleGraph + PathHandleGraph,
    T: OptFields,
{
    let mut graph: G = Default::default();

    for segment in gfa.segments.iter() {
        assert!(segment.name > 0);
        let seq = &segment.sequence;
        graph.create_handle(seq, segment.name);
    }

    for link in gfa.links.iter() {
        let left = Handle::new(link.from_segment, link.from_orient);
        let right = Handle::new(link.from_segment, link.from_orient);
        graph.create_edge(Edge(left, right));
    }

    for path in gfa.paths.iter() {
        let name = &path.path_name;
        let path_id = graph.create_path_handle(name, false);
        for (seg, orient) in path.iter() {
            let handle = Handle::new(seg, orient);
            graph.append_step(&path_id, handle);
        }
    }

    graph
}

pub fn fill_gfa_lines<G, I, T>(graph: &mut G, gfa_lines: I) -> GFAResult<()>
where
    G: AdditiveHandleGraph + PathHandleGraph,
    I: Iterator<Item = GFAResult<Line1<usize, T>>>,
    T: OptFields,
{
    for line in gfa_lines {
        let line = line?;
        match line {
            Line1::Segment(v) => {
                let id = NodeId::from(v.name);
                graph.create_handle(&v.sequence, id);
            }
            Line1::Link(v) => {
                let left = Handle::new(v.from_segment, v.from_orient);
                let right = Handle::new(v.to_segment, v.to_orient);
                graph.create_edge(Edge(left, right));
            }
            Line1::Path(v) => {
                let name = &v.path_name;
                let path_id = graph.create_path_handle(name, false);
                for (seg, orient) in v.iter() {
                    let handle = Handle::new(seg, orient);
                    graph.append_step(&path_id, handle);
                }
            }
            _ => (),
        }
    }

    Ok(())
}

/// Function that takes a HashGraph object as input and return a GFA object
/// This function is still ```Work In Progress``` so it's not perfect.\
/// Sometimes can leads to unexpected bugs.
/// # Example
/// ```ignore
/// let parser = GFAParser::new();
/// let gfa_in: GFA<usize, ()> = parser.parse_file("./tests/gfa2_files/file.gfa").unwrap();
///
/// let graph = HashGraph::from_gfa(&gfa_in);
/// let gfa_out: GFA<usize, ()> = handlegraph2::conversion::to_gfa(&graph);
///
/// println!("{}", gfa_out);
/// println!("{}", gfa_in);
///  
/// /* hashgraph to gfa:
/// H   VN:Z:1.0
/// S   13  CTTGATT
/// S   12  TCAAGG
/// S   11  ACCTT
/// L   12  -   13  +   0M
/// L   11  +   12  -   0M
/// L   11  +   13  +   0M
/// P   14  11+ 12- 13+ 0M
/// */
/// 
/// /* original gfa:
/// H   VN:Z:1.0
/// S   13  CTTGATT
/// S   12  TCAAGG
/// S   11  ACCTT
/// L   12  -   13  +   0M
/// L   11  +   12  -   0M
/// L   11  +   13  +   0M
/// P   14  11+ 12- 13+ 0M
/// */
/// ```
pub fn to_gfa(graph: &HashGraph) -> GFA<usize, ()> {
    use crate::handlegraph::*;
    let mut gfa = GFA::new();

    for handle in graph.all_handles() {
        let name = usize::from(handle.id());
        let sequence: BString = graph.sequence_iter(handle.forward()).collect();

        let segment = Segment1 {
            name,
            sequence,
            optional: (),
        };
        gfa.segments.push(segment);
    }

    let orient = |rev: bool| {
        if rev {
            Orientation::Backward
        } else {
            Orientation::Forward
        }
    };

    for edge in graph.all_edges() {
        let Edge(left, right) = edge;
        let from_segment: usize = usize::from(left.id());
        let from_orient = orient(left.is_reverse());
        let to_segment: usize = usize::from(right.id());
        let to_orient = orient(right.is_reverse());
        let overlap = BString::from("0M");

        let link = Link {
            from_segment,
            from_orient,
            to_segment,
            to_orient,
            overlap,
            optional: (),
        };

        gfa.links.push(link);
    }

    for path_id in graph.paths_iter() {
        let path_name: BString = graph.path_handle_to_name(path_id).into();
        let mut segment_names: Vec<Vec<u8>> = Vec::new();
        for step in graph.steps_iter(path_id) {
            let handle = graph.handle_of_step(&step).unwrap();
            let segment: usize = handle.id().into();
            let orientation = orient(handle.is_reverse());
            segment_names.push(segment.to_string().into());
            segment_names.push(orientation.to_string().into());
            segment_names.push(",".into());
        }
        let segment_names: BString =
            segment_names.into_iter().flatten().collect();

        let path: Path<usize, ()> =
            Path::new(path_name, segment_names, "0M".into(), ());

        gfa.paths.push(path);
    }

    gfa
}
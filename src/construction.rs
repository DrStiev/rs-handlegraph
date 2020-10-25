use crate::{
    handle::{Edge, Handle, NodeId},
    handlegraph::HandleGraph,
    mutablehandlegraph::MutableHandleGraph,
    pathgraph::PathHandleGraph,
};

use gfa2::{
    gfa2::{Line, GFA2, orientation::Orientation},
    tag::OptFields,
    parser_gfa2::GFA2Result,
};

pub fn from_gfa<G, T>(gfa2: &GFA2<usize, T>) -> G
where
    G: Default + MutableHandleGraph + PathHandleGraph,
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
            "+" => Orientation::Forward,
            "-" => Orientation::Backward,
            _ => panic!("Error! Edge did not include orientation"),
        };
        let right_orient = match &link.sid2.to_string()[right_len-1..] {
            "+" => Orientation::Forward,
            "-" => Orientation::Backward,
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

pub fn fill_gfa_lines<G, I, T>(graph: &mut G, gfa_lines: I) -> GFA2Result<()>
where
    G: MutableHandleGraph + PathHandleGraph,
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
                    "+" => Orientation::Forward,
                    "-" => Orientation::Backward,
                    _ => panic!("Error! Edge did not include orientation"),
                };
                let right_orient = match &v.sid2.to_string()[right_len-1..] {
                    "+" => Orientation::Forward,
                    "-" => Orientation::Backward,
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

# Rust Handlegraph
Variation graphs in Rust, based on the C++ library
[libhandlegraph](https://github.com/vgteam/libhandlegraph).

While this draws heavily on the C++ implementation for now,
compatibility is not a goal, and the API will surely diverge as
development proceeds.\
This library it's a variation of the library developed by **Christian Fischer** [link here](https://github.com/chfi/rs-handlegraph).

## Usage
This library performs 4 main operations:
- Given a GFA (or GFA2) object it creates the corresponding HashGraph.\
To perform this operation the GFA Object NEEDS to be in [usize](https://doc.rust-lang.org/std/primitive.usize.html) type and usually the OptionalFields field of the GFA object it's ignored.
```rust
let parser: GFA2Parser<usize, ()> = GFA2Parser::new();
let gfa2: GFA2<usize, ()> = parser
    .parse_file(&"./tests/gfa2_files/spec_q7.gfa")
    .unwrap();
println!("{:#?}", gfa2);
```
The GFA Object will look like:
```
GFA2 {
    headers: [
        Header {
            version: Some("VN:Z:2.0"),
            tag: (),
        },
        Header {
            version: Some("ul:Z:https://github.com/pmelsted/GFA-spec/issues/7#issuecomment-219685552"),
            tag: (),
        },
    ],
    segments: [
        Segment {
            id: 11,
            len: "5",
            sequence: "ACCTT",
            tag: (),
        },
        Segment {
            id: 12,
            len: "6",
            sequence: "TCAAGG",
            tag: (),
        },
        Segment {
            id: 13,
            len: "7",
            sequence: "CTTGATT",
            tag: (),
        },
    ],
    fragments: [],
    edges: [
        Edge {
            id: 42,
            sid1: 110,
            sid2: 121,
            beg1: "1",
            end1: "5$",
            beg2: "2",
            end2: "6$",
            alignment: "4M",
            tag: (),
        },
        Edge {
            id: 42,
            sid1: 121,
            sid2: 130,
            beg1: "0",
            end1: "5",
            beg2: "0",
            end2: "5",
            alignment: "5M",
            tag: (),
        },
        Edge {
            id: 42,
            sid1: 110,
            sid2: 130,
            beg1: "2",
            end1: "5$",
            beg2: "0",
            end2: "3",
            alignment: "3M",
            tag: (),
        },
    ],
    gaps: [],
    groups_o: [
        GroupO {
            id: "14",
            var_field: "11+ 12- 13+",
            tag: (),
            _segment_names: PhantomData,
        },
    ],
    groups_u: [],
}
```
The only fields used to create the graph are:
1. Segment fields
2. Edge (or Link) fields
3. O-Group (or Path) fields
4. The U-Group fields are used to create the graph but it's still WIP
```rust
let graph = HashGraph::from_gfa2(&gfa);
println!("{:#?}", graph);
``` 
The Graph object will look like:
```
HashGraph {
max_id: NodeId(13),
min_id: NodeId(11),
    graph: {
        NodeId(13): Node {
            sequence: "CTTGATT",
            left_edges: [
                Handle(24),
                Handle(23),
            ],
            right_edges: [],
            occurrences: {0: 2},
        },
        NodeId(12): Node {
            sequence: "TCAAGG",
            left_edges: [
                Handle(26),
            ],
            right_edges: [
                Handle(23),
            ],
            occurrences: {0: 1},
        },
        NodeId(11): Node {
            sequence: "ACCTT",
            left_edges: [],
            right_edges: [
                Handle(25),
                Handle(26),
            ],
            occurrences: {0: 0},
        },
    },
    path_id: {
        [49,52]: 0
    },
    paths: {
        0: Path {
            path_id: 0,
            name: "14",
            is_circular: false,
            nodes: [
                Handle(22),
                Handle(25),
                Handle(26),
            ],
        },
    },
}
```
- Given an HashGraph object it creates the corresponding GFA (GFA2) Object.\
Because the resulting GFA Object will use the [BString](https://docs.rs/bstr/0.2.14/bstr/struct.BString.html) type instead of the usize, the Orientation field will be reverted from [01] to [+-], but all the other fields will be kept as number instead of finding their old string value.
```rust
let parser = GFA2Parser::new();
let gfa_in: GFA2<usize, ()> = parser.parse_file("./tests/gfa2_files/spec_q7.gfa").unwrap();
let graph: HashGraph = HashGraph::from_gfa2(&gfa_in);
let _gfa_out: GFA2<BString, ()> = to_gfa2(&graph);
println!("{:#?}", _gfa_out);
```
The resulting object will look like
```
GFA2 {
    headers: [
        Header {
            version: Some("VN:Z:2.0"),
            tag: (),
        },
    ],
    segments: [
        Segment {
            id: "13",
            len: "7",
            sequence: "CTTGATT",
            tag: (),
        },
        Segment {
            id: "12",
            len: "6",
            sequence: "TCAAGG",
            tag: (),
        },
        Segment {
            id: "11",
            len: "5",
            sequence: "ACCTT",
            tag: (),
        },
    ],
    fragments: [],
    edges: [
        Edge {
            id: "*",
            sid1: "12-",
            sid2: "13+",
            beg1: "0",
            end1: "0$",
            beg2: "0",
            end2: "0$",
            alignment: "0M",
            tag: (),
        },
        Edge {
            id: "*",
            sid1: "11+",
            sid2: "12-",
            beg1: "0",
            end1: "0$",
            beg2: "0",
            end2: "0$",
            alignment: "0M",
            tag: (),
        },
        Edge {
            id: "*",
            sid1: "11+",
            sid2: "13+",
            beg1: "0",
            end1: "0$",
            beg2: "0",
            end2: "0$",
            alignment: "0M",
            tag: (),
        },
    ],
    gaps: [],
    groups_o: [
        GroupO {
            id: "14",
            var_field: "11+ 12- 13+",
            tag: (),
            _segment_names: PhantomData,
        },
    ],
    groups_u: [],
}
```
- Given an HashGraph this can be "pretty-printed" to make it easier to understand:
```rust
let graph: HashGraph = HashGraph::from_gfa2(&gfa_in);
graph.print_graph();
```
Obtaining:
```
Graph: {
        Nodes: {
                13: CTTGATT
                12: TCAAGG
                11: ACCTT
        }
        Edges: {
                12- --> 13+
                11+ --> 12-
                11+ --> 13+
        }
        Paths: {
                14: ACCTT -> CCTTGA -(TCAAGG) -> CTTGATT
        }
}
```
- Given an HashGaph it's possible to ADD, REMOVE and MODIFY the values in it:
    - ADD OPERATIONS: 
    ```rust
    fn append_handle(&mut self, seq: &[u8]) -> Handle;

    fn create_handle<T: Into<NodeId>>(&mut self, seq: &[u8], node_id: T) -> Handle;
    
    fn create_edge(&mut self, edge: Edge) -> bool;
    ```
    - REMOVE OPERATIONS:
    ```rust
    fn remove_handle<T: Into<NodeId>>(&mut self, node: T) -> bool;
    
    fn remove_edge(&mut self, edge: Edge) -> bool;
    
    fn remove_path(&mut self, name: &[u8]) -> bool;
    
    fn clear_graph(&mut self);
    ```
    - MODIFIY OPERATIONS:
    ```rust
    fn modify_handle<T: Into<NodeId>>(&mut self, node_id: T, seq: &[u8]) -> bool;
    
    fn modify_edge(
        &mut self,
        old_edge: Edge,
        left_node: Option<Handle>,
        right_node: Option<Handle>,
    ) -> bool;
    
    fn modify_path(&mut self, path_name: &[u8], sequence_of_id: Vec<Handle>) -> bool;
    ```

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
    // create a new handle.
    // an Handle is a NodeId with and Orientation packed as a single u64 
    let h1 = graph.create_handle(b"1", 1);
    let h2 = graph.create_handle(b"2", 2);

    // create an edge between 2 handles
    graph.create_edge(Edge(h1, h2));

    // create a new path specifying if it circular or not
    let p1 = graph.create_path_handle(b"path-1", false);
    // insert in the path created the "steps" (handle)
    graph.append_step(&p1, h1);
    graph.append_step(&p1, h2);
    ```
    Obtaining
    ```
    Graph: {
        Nodes: {
                1: 1
                2: 2
                13: CTTGATT
                12: TCAAGG
                11: ACCTT
        }
        Edges: {
                1+ --> 2+
                12- --> 13+
                11+ --> 12-
                11+ --> 13+
        }
        Paths: {
                path-1: 1 -> 2
                15: ACCTT -> CTTGATT   
                14: ACCTT -> CCTTGA -(TCAAGG) -> CTTGATT
        }
    }
    ```
    - REMOVE OPERATIONS:
    ```rust
    let remove_id: NodeId = 12.into();
    // remove the node if exists and all its's occurrencies from the edge and path list
    graph.remove_handle(remove_id);
    ```
    Obtaining
    ``` 
    Graph: {
        Nodes: {
                13: CTTGATT
                11: ACCTT
        }
        Edges: {
                11+ --> 13+
        }
        Paths: {
                15: ACCTT -> CTTGATT   
        }
    }
    ```
    ```rust
    let left: Handle = Handle::new(12 as u64, Orientation::Backward);
    let right: Handle = Handle::new(13 as u64, Orientation::Forward);
    let remove_edge: Edge = Edge(left, right);
    // remove the edge if exists, and all it's occurrencies from the path list
    graph.remove_edge(remove_edge);
    ```
    Obtaining
    ```
    Graph: {
        Nodes: {
                13: CTTGATT
                12: TCAAGG
                11: ACCTT
        }
        Edges: {
                11+ --> 12-
                11+ --> 13+
        }
        Paths: {
                15: ACCTT -> CTTGATT   
        }
    }
    ```
    ```rust
    // remove a path if exists
    graph.remove_path(&BString::from(15.to_string()));
    ```
    Obtaining
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
    ```rust
    // delete all the occurrencies in the graph and set te max and min nodes
    // values to default
    graph.clear_graph();
    ```
    Obtaining
    ``` 
    Graph: {
        Nodes: {
        }
        Edges: {
        }
        Paths: {
        }
    }
    ```
    - MODIFIY OPERATIONS:
    ```rust
    let modify_id: NodeId = 12.into();
    let modify_seq: &[u8] = b"TEST_SEQUENCE";
    // modify a node if exists, changing its sequence and leaving the nodeId the same
    // the new sequence will replace all the occurrencies of the old sequence in the path vector
    graph.modify_handle(modify_id, modify_seq);
    ```
    Obtaining
    ```
    Graph: {
        Nodes: {
                13: CTTGATT
                12: TEST_SEQUENCE      
                11: ACCTT
        }
        Edges: {
                12- --> 13+
                11+ --> 12-
                11+ --> 13+
        }
        Paths: {
                15: ACCTT -> CTTGATT   
                14: ACCTT -> EGNEUQES_ASEA -(TEST_SEQUENCE) -> CTTGATT        
        }
    }
    ```
    ```rust
    let left: Handle = Handle::new(11 as u64, Orientation::Forward);
    let right: Handle = Handle::new(13 as u64, Orientation::Forward);
    let new_left: Handle = Handle::new(11 as u64, Orientation::Forward);
    let new_right: Handle = Handle::new(11 as u64, Orientation::Forward);
    let mod_edge: Edge = Edge(left, right);
    // modify an edge if exists, removing all the occurrencies of the edge from the path list
    graph.modify_edge(mod_edge, new_left, new_right)
    ```
    Obtaining
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
                11+ --> 11+
        }
        Paths: { 
                14: ACCTT -> CCTTGA -(TCAAGG) -> CTTGATT
        }
    }
    ```
    ```rust
    let path_handles: Vec<Handle> = vec![
        Handle::new(11, Orientation::Forward),
        Handle::new(11, Orientation::Forward),
        Handle::new(12, Orientation::Forward),
        Handle::new(13, Orientation::Forward),
    ];
    // modify a path if exists, replace all the old occurrencies with the new ones
    graph.modify_path(b"14", path_handles);
    ```
    Obtaining
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
                15: ACCTT -> CTTGATT   
                14: ACCTT -> ACCTT -> TCAAGG -> CTTGATT
        }
    }
    ```

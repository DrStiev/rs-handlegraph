use crate::handle::{Edge, Handle, NodeId};
use crate::hashgraph::Path;
use crate::handlegraph::{HandleGraph, HandleGraphRef};

pub trait SubtractiveHandleGraph {
    /// Function that remove a node and all its occurrencies 
    /// # Example
    /// ```ignore
    /// // Nodes: 11, 12, 13
    /// // Edges: 11 -> 12, 11 -> 13, 12 -> 13
    /// 
    /// let remove_id: NodeId = 12.into();
    /// graph.remove_handle(remove_id);
    /// 
    /// // Nodes: 11, 13
    /// // Edges: 11 -> 13
    /// ```
    fn remove_handle<T: Into<NodeId>>(&mut self, node: T) -> bool;

    /// Function that removes an Edge (Link) between 2 nodes
    /// # Example
    /// ```ignore
    /// // Nodes: 11, 12, 13
    /// // Edges: 11 -> 12, 11 -> 13, 12 -> 13
    /// 
    /// let h1: NodeId = 11.into();
    /// let h3: NodeId = 13.into();
    /// graph.remove_edge(Edge(h1, h3));
    /// 
    /// // Nodes: 11, 12, 13
    /// // Edges: 11 -> 12, 12 -> 13
    /// ```
    fn remove_edge(&mut self, edge: Edge) -> bool;

    /// Function that removes an Edge (Link) between 2 nodes
    /// # Example
    /// ```ignore
    /// // Nodes: 11, 12, 13
    /// // Edges: 11 -> 12, 11 -> 13, 12 -> 13
    /// // Path: 0 (14): 11 -> 12 -> 13, 1 (15): 11 -> 13
    /// graph.remove_path(&BString::from(15.to_string()));
    /// 
    /// // Nodes: 11, 12, 13
    /// // Edges: 11 -> 12, 11 -> 13, 12 -> 13
    /// // Path: 0 (14): 11 -> 12 -> 13
    /// ```
    fn remove_path(&mut self, name: &[u8]) -> bool;

    /// Function that clears the graph and set max_id to 0 and min_id to u64::MAX
    /// like the Default implementation fore HashGraph
    fn clear_graph(&mut self);
}

pub trait AdditiveHandleGraph {
    fn append_handle(&mut self, seq: &[u8]) -> Handle;

    fn create_handle<T: Into<NodeId>>(
        &mut self,
        seq: &[u8],
        node_id: T,
    ) -> Handle;

    fn create_edge(&mut self, edge: Edge);
}

pub trait ModdableHandleGraph {
    /*
    fn divide_handle(
        &mut self,
        handle: Handle,
        offsets: Vec<usize>,
    ) -> Vec<Handle>;

    fn split_handle(
        &mut self,
        handle: Handle,
        offset: usize,
    ) -> (Handle, Handle) {
        let handles = self.divide_handle(handle, vec![offset]);
        (handles[0], handles[1])
    }

    fn apply_orientation(&mut self, handle: Handle) -> Handle;
    */

    /// given a node, this function will replace the sequence associated to the NodeId
    fn modify_handle<T: Into<NodeId>>(
        &mut self, 
        node_id: T, 
        seq: &[u8]
    ) -> bool;
    
    /// given an Edge, this function will replace the left, or the right NodeId
    /// it can even replace the right and left Nodes
    fn modify_edge(
        &mut self, 
        old_edge: Edge, 
        left_node: Option<Handle>, 
        right_node: Option<Handle>,
    ) -> bool;

    /// given a pathname, this function will replace the sequence of ids
    fn modify_path(
        &mut self,
        path_name: &[u8],
        sequence_of_id: Vec<Handle>,
    ) -> bool;
}

/// Trait encapsulating the mutable aspects of a handlegraph
/// WIP
pub trait MutableHandleGraph: HandleGraph {
    /*
    fn append_handle(&mut self, seq: &[u8]) -> Handle;
    fn create_handle<T: Into<NodeId>>(
        &mut self,
        seq: &[u8],
        node_id: T,
    ) -> Handle;
    fn create_edge(&mut self, edge: Edge);
    */

    fn divide_handle(
        &mut self,
        handle: Handle,
        offsets: Vec<usize>,
    ) -> Vec<Handle>;

    fn split_handle(
        &mut self,
        handle: Handle,
        offset: usize,
    ) -> (Handle, Handle) {
        let handles = self.divide_handle(handle, vec![offset]);
        (handles[0], handles[1])
    }

    fn apply_orientation(&mut self, handle: Handle) -> Handle;
}

pub trait MutHandleGraphRef: HandleGraphRef {}

impl<'a, T> MutHandleGraphRef for &'a T
where
    T: HandleGraph,
    &'a T: HandleGraphRef,
{
}
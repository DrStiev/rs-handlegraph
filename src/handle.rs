use gfa::gfa::Orientation;
use std::cmp::Ordering;
use std::ops::Add;

/// Newtype that represents a node in the graph, no matter the
/// graph implementation
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct NodeId(u64);

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for NodeId {
    fn from(num: u64) -> Self {
        NodeId(num)
    }
}

impl From<NodeId> for u64 {
    fn from(id: NodeId) -> Self {
        id.0
    }
}

impl Add<u64> for NodeId {
    type Output = Self;

    fn add(self, other: u64) -> Self {
        NodeId(self.0 + other)
    }
}

/// A Handle is a node ID with an orientation, packed as a single u64
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq, Ord)]
#[repr(transparent)]
pub struct Handle(u64);

impl Handle {
    pub fn as_integer(self) -> u64 {
        self.0
    }

    pub const fn from_integer(i: u64) -> Self {
        Handle(i)
    }

    pub fn unpack_number(self) -> u64 {
        self.as_integer() >> 1
    }

    pub fn unpack_bit(self) -> bool {
        self.as_integer() & 1 != 0
    }

    pub fn new<T: Into<u64>>(id: T, orient: Orientation) -> Handle {
        let id: u64 = id.into();
        let is_reverse = if orient == Orientation::Forward {
            false
        } else {
            true
        };
        if id < (0x1 << 63) {
            Handle::from_integer((id << 1) | is_reverse as u64)
        } else {
            panic!(
                "Tried to create a handle with a node ID that filled 64 bits"
            )
        }
    }

    pub fn pack<T: Into<u64>>(id: T, is_reverse: bool) -> Handle {
        let id: u64 = id.into();
        if id < (0x1 << 63) {
            Handle::from_integer((id << 1) | is_reverse as u64)
        } else {
            panic!(
                "Tried to create a handle with a node ID that filled 64 bits"
            )
        }
    }

    pub fn id(self) -> NodeId {
        NodeId(self.unpack_number())
    }

    pub fn is_reverse(&self) -> bool {
        self.unpack_bit()
    }

    pub fn flip(self) -> Self {
        Handle(self.as_integer() ^ 1)
    }

    pub fn forward(self) -> Self {
        if self.is_reverse() {
            self.flip()
        } else {
            self
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct Edge(pub Handle, pub Handle);

impl Edge {
    /// Construct an edge, taking the orientation of the handles into account
    pub fn edge_handle(left: Handle, right: Handle) -> Edge {
        let flipped_right = right.flip();
        let flipped_left = left.flip();

        match left.cmp(&flipped_right) {
            Ordering::Greater => Edge(flipped_right, flipped_left),
            Ordering::Equal => {
                if right > flipped_left {
                    Edge(flipped_right, flipped_left)
                } else {
                    Edge(left, right)
                }
            }
            Ordering::Less => Edge(left, right),
        }
    }
}

/// Enum for handle orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction {
    Left,
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Handle::pack is an isomorphism; Handle <=> (u63, bool)
    #[test]
    fn handle_is_isomorphism() {
        let u: u64 = 597283742;
        let h = Handle::pack(NodeId(u), true);
        assert_eq!(h.unpack_number(), u);
        assert_eq!(h.unpack_bit(), true);
    }

    // Handle::pack should panic when the provided NodeId is invalid
    // (i.e. uses the 64th bit
    #[test]
    #[should_panic]
    fn handle_pack_panic() {
        Handle::pack(NodeId(std::u64::MAX), true);
    }

    #[test]
    fn handle_flip() {
        let u: u64 = 597283742;
        let h1 = Handle::pack(NodeId(u), true);
        let h2 = h1.flip();

        let h3 = Handle::pack(NodeId(u), false);
        println!("{:?}, {}, {}", h1, h1.unpack_bit(), h1.is_reverse());
        println!("{:?}, {}, {}", h2, h2.unpack_bit(), h2.is_reverse());
        println!("{:?}, {}, {}", h3, h3.unpack_bit(), h3.is_reverse());

        assert_eq!(h1.unpack_number(), h2.unpack_number());
        assert_eq!(h1.unpack_bit(), true);
        assert_eq!(h2.unpack_bit(), false);
    }
}

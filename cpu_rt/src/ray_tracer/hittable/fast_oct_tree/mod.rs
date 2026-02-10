mod arena;
mod hit_info;
mod hittable;
mod leafable;
mod operations;
mod prelude;
mod ray_trace;
mod shapes;
mod stats;
mod voxel;
use cgmath::Point3;
use leafable::Leafable;

pub type IndexType = u32;
pub type TreePosition = Point3<IndexType>;
use arena::{Arena, ArenaIndex};

pub(crate) use arena::ArenaStats;
pub(crate) use stats::FastOctTreeStats;
pub(crate) use voxel::{SolidVoxel, VolumeVoxel, Voxel, VoxelMaterial};

#[derive(Clone, Debug, PartialEq)]
enum NodeData<T: Leafable> {
    Parent { children: [ArenaIndex; 8] },
    Leaf(T),
    Empty,
}
#[derive(Clone, Debug, PartialEq)]
struct Node<T: Leafable> {
    data: NodeData<T>,
    // size is expressed in powers of 2, so if self.size = 2 the voxel size will be 2.pow(2) or 4 meters
    size: u32,
}
impl<T: Leafable> Node<T> {
    fn is_leaf(&self) -> bool {
        match self.data {
            NodeData::Leaf(_) => true,
            _ => false,
        }
    }
    fn is_empty(&self) -> bool {
        match self.data {
            NodeData::Empty => true,
            _ => false,
        }
    }
    //sets children and returns  a copy of modified version of self
    fn set_child(mut self, value: T, position: TreePosition, arena: &mut Arena<Self>) -> Self {
        if self.size == 0 {
            assert_eq!(position, Point3::new(0, 0, 0));
            self.data = NodeData::Leaf(value);

            self
        } else {
            match self.data {
                // if self is not a parent setting value to parent
                NodeData::Empty => {
                    let children = [
                        arena.insert(Node {
                            data: NodeData::Empty,
                            size: self.size - 1,
                        }),
                        arena.insert(Node {
                            data: NodeData::Empty,
                            size: self.size - 1,
                        }),
                        arena.insert(Node {
                            data: NodeData::Empty,
                            size: self.size - 1,
                        }),
                        arena.insert(Node {
                            data: NodeData::Empty,
                            size: self.size - 1,
                        }),
                        arena.insert(Node {
                            data: NodeData::Empty,
                            size: self.size - 1,
                        }),
                        arena.insert(Node {
                            data: NodeData::Empty,
                            size: self.size - 1,
                        }),
                        arena.insert(Node {
                            data: NodeData::Empty,
                            size: self.size - 1,
                        }),
                        arena.insert(Node {
                            data: NodeData::Empty,
                            size: self.size - 1,
                        }),
                    ];
                    self.data = NodeData::Parent { children };
                }
                NodeData::Leaf(leaf) => {
                    let children = [
                        arena.insert(Node {
                            data: NodeData::Leaf(leaf.clone()),
                            size: self.size - 1,
                        }),
                        arena.insert(Node {
                            data: NodeData::Leaf(leaf.clone()),
                            size: self.size - 1,
                        }),
                        arena.insert(Node {
                            data: NodeData::Leaf(leaf.clone()),
                            size: self.size - 1,
                        }),
                        arena.insert(Node {
                            data: NodeData::Leaf(leaf.clone()),
                            size: self.size - 1,
                        }),
                        arena.insert(Node {
                            data: NodeData::Leaf(leaf.clone()),
                            size: self.size - 1,
                        }),
                        arena.insert(Node {
                            data: NodeData::Leaf(leaf.clone()),
                            size: self.size - 1,
                        }),
                        arena.insert(Node {
                            data: NodeData::Leaf(leaf.clone()),
                            size: self.size - 1,
                        }),
                        arena.insert(Node {
                            data: NodeData::Leaf(leaf),
                            size: self.size - 1,
                        }),
                    ];
                    self.data = NodeData::Parent { children };
                }
                NodeData::Parent { .. } => {}
            };

            match self.data {
                NodeData::Parent { children } => {
                    let index = Node::<T>::world_pos_to_child_index(position, self.size) as usize;
                    let pos_in_child = Self::world_pos_to_child_pos(position, self.size);

                    let child = arena.get(children[index]).expect("should exist").clone();

                    let child_clone = child.clone().set_child(value, pos_in_child, arena);

                    arena.update(children[index], child_clone.clone());

                    let child0 = arena.get(children[0]).unwrap().clone();
                    let child1 = arena.get(children[1]).unwrap().clone();
                    let child2 = arena.get(children[2]).unwrap().clone();
                    let child3 = arena.get(children[3]).unwrap().clone();
                    let child4 = arena.get(children[4]).unwrap().clone();
                    let child5 = arena.get(children[5]).unwrap().clone();
                    let child6 = arena.get(children[6]).unwrap().clone();
                    let child7 = arena.get(children[7]).unwrap().clone();
                    if child0 == child1
                        && child0 == child2
                        && child0 == child3
                        && child0 == child4
                        && child0 == child5
                        && child0 == child6
                        && child0 == child7
                        && (child0.is_leaf() || child0.is_empty())
                    {
                        self.data = child0.data.clone();
                        for child in children {
                            arena.delete(child);
                        }
                        self
                    } else {
                        self
                    }
                }
                _ => panic!("must be parent"),
            }
        }
    }

    fn get(&self, position: TreePosition, arena: &Arena<Self>) -> Option<T> {
        match &self.data {
            NodeData::Empty => None,
            NodeData::Leaf(leaf) => Some(leaf.clone()),
            NodeData::Parent { children } => {
                let child_index = Self::world_pos_to_child_index(position, self.size);
                let child = arena
                    .get(children[child_index as usize])
                    .expect("should have child")
                    .clone();
                let output = child.get(Self::world_pos_to_child_pos(position, self.size), arena);
                output
            }
        }
    }
    const fn empty() -> Self {
        Self::empty_size(0)
    }
    const fn empty_size(size: u32) -> Self {
        Self {
            data: NodeData::Empty,
            size,
        }
    }
    /// returns the index of the position
    const fn pos_to_index(position: TreePosition) -> usize {
        match position {
            Point3 { x: 0, y: 0, z: 0 } => 0,
            Point3 { x: 0, y: 0, z: 1 } => 1,
            Point3 { x: 0, y: 1, z: 0 } => 2,
            Point3 { x: 0, y: 1, z: 1 } => 3,
            Point3 { x: 1, y: 0, z: 0 } => 4,
            Point3 { x: 1, y: 0, z: 1 } => 5,
            Point3 { x: 1, y: 1, z: 0 } => 6,
            Point3 { x: 1, y: 1, z: 1 } => 7,
            _ => panic!("unsupported position"),
        }
    }
    const fn index_to_pos(index: usize) -> TreePosition {
        match index {
            0 => Point3::new(0, 0, 0),
            1 => Point3::new(0, 0, 1),
            2 => Point3::new(0, 1, 0),
            3 => Point3::new(0, 1, 1),
            4 => Point3::new(1, 0, 0),
            5 => Point3::new(1, 0, 1),
            6 => Point3::new(1, 1, 0),
            7 => Point3::new(1, 1, 1),
            _ => panic!("unsupported index"),
        }
    }
    const fn world_pos_to_child_pos(position: TreePosition, size: u32) -> TreePosition {
        let mask = !(1 << (size - 1));
        Point3 {
            x: position.x & mask,
            y: position.y & mask,
            z: position.z & mask,
        }
    }
    const fn world_pos_to_child_index(position: TreePosition, size: u32) -> u32 {
        let child_size = size - 1;

        Self::pos_to_index(Point3 {
            x: position.x >> child_size,
            y: position.y >> child_size,
            z: position.z >> child_size,
        }) as u32
    }
    fn get_world_size(&self) -> IndexType {
        1 << self.size
    }
}

#[derive(Clone, Debug)]
/// Overall Tree data structure. Utilizes arena to maintain cache locality and to serve as a framework as I migrate towards GPU compute
pub(crate) struct FastOctTree<T: Leafable> {
    arena: Arena<Node<T>>,
}
impl<T: Leafable> FastOctTree<T> {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
        }
    }
    /// gets the size of the node in world units
    fn world_size(&self) -> IndexType {
        if let Some(root) = self.arena.get_root_ref() {
            root.get_world_size()
        } else {
            0
        }
    }

    pub fn get(&self, position: TreePosition) -> Option<T> {
        if let Some(root) = self.arena.get_root_ref() {
            let world_size = root.get_world_size();
            if position.x < world_size && position.y < world_size && position.z < world_size {
                root.get(position, &self.arena)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn new() {
        let _ = FastOctTree::<u32>::new();
    }
    #[test]
    fn get_empty() {
        let t = FastOctTree::<u32>::new();
        assert_eq!(t.get(Point3::new(0, 0, 0)), None);
    }

    #[test]
    fn size_empty() {
        let t = FastOctTree::<u32>::new();
        assert_eq!(t.world_size(), 0)
    }
    #[test]
    fn size_one() {
        let mut t = FastOctTree::new();
        t.set(0u32, Point3::new(0, 0, 0));
        assert_eq!(t.get(Point3::new(0, 0, 0)).unwrap(), 0);
        assert_eq!(t.world_size(), 1)
    }
    #[test]
    fn pos_and_index() {
        let positions = [
            Point3::new(0, 0, 0),
            Point3::new(0, 0, 1),
            Point3::new(0, 1, 0),
            Point3::new(0, 1, 1),
            Point3::new(1, 0, 0),
            Point3::new(1, 0, 1),
            Point3::new(1, 1, 0),
            Point3::new(1, 1, 1),
        ];

        let indices = [0, 1, 2, 3, 4, 5, 6, 7];
        for (position, index) in positions.iter().zip(indices) {
            assert_eq!(Node::<u32>::pos_to_index(*position), index);
            assert_eq!(Node::<u32>::index_to_pos(index), *position)
        }
    }
    #[test]
    fn child_world_pos() {
        assert_eq!(
            Node::<u32>::world_pos_to_child_pos(Point3::new(0, 0, 0), 2),
            Point3::new(0, 0, 0)
        );
        assert_eq!(
            Node::<u32>::world_pos_to_child_pos(Point3::new(3, 0, 0), 2),
            Point3::new(1, 0, 0)
        );
        assert_eq!(
            Node::<u32>::world_pos_to_child_pos(Point3::new(3, 0, 0), 3),
            Point3::new(3, 0, 0)
        );
    }
    #[test]
    fn child_world_index() {
        assert_eq!(
            Node::<u32>::world_pos_to_child_index(Point3::new(0, 0, 0), 2),
            0
        );
        assert_eq!(
            Node::<u32>::world_pos_to_child_index(Point3::new(0, 0, 3), 2),
            1
        );
        assert_eq!(
            Node::<u32>::world_pos_to_child_index(Point3::new(0, 3, 1), 2),
            2
        );
        assert_eq!(
            Node::<u32>::world_pos_to_child_index(Point3::new(0, 3, 3), 2),
            3
        );
    }
}

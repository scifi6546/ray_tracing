use super::{prelude::*, LeafType, Leafable, OctTreeChildren};
use cgmath::Point3;
use log::info;
#[derive(Clone, Debug)]
pub(crate) struct OctTreeNode<T: Leafable> {
    pub children: OctTreeChildren<T>,
    pub size: u32,
}
impl<T: Leafable> OctTreeNode<T> {
    pub fn is_optimal(&self, debug_print: bool) -> bool {
        match &self.children {
            OctTreeChildren::Leaf(_) => true,
            OctTreeChildren::ParentNode(children) => {
                let mut val = match &children[0].children {
                    OctTreeChildren::Leaf(val) => Some(val),
                    OctTreeChildren::ParentNode(_) => None,
                };
                if val.is_some() {
                    for i in 1..8 {
                        match &children[i].children {
                            OctTreeChildren::Leaf(val2) => {
                                if Some(val2) != val {
                                    val = None;
                                    break;
                                }
                            }
                            OctTreeChildren::ParentNode(_) => {
                                val = None;
                                break;
                            }
                        }
                    }
                }
                if debug_print {
                    info!("node not optimal size: {:#?}", self.size);
                }
                if val.is_some() {
                    false
                } else {
                    children
                        .iter()
                        .map(|c| c.is_optimal(debug_print))
                        .fold(true, |acc, x| acc && x)
                }
            }
        }
    }
    /// returns ray in distance it hit

    pub fn get_child_index(&self, x: u32, y: u32, z: u32) -> usize {
        let x_v = x / (self.size / 2);
        let y_v = y / (self.size / 2);
        let z_v = z / (self.size / 2);
        get_child_index_size2(x_v, y_v, z_v)
    }
    /// gets the size given self size is 2

    pub fn get(&self, pos: Point3<u32>) -> &LeafType<T> {
        match &self.children {
            OctTreeChildren::Leaf(val) => val,
            OctTreeChildren::ParentNode(children) => {
                let idx = self.get_child_index(pos.x, pos.y, pos.z);
                if idx >= children.len() {
                    println!("idx: {}, x: {}, y: {}, z: {}", idx, pos.x, pos.y, pos.z);
                }

                children[idx].get(pos.map(|v| v % (self.size / 2)))
            }
        }
    }
}
impl<T: Leafable> OctTreeNode<T> {
    pub(crate) fn is_leaf(&self) -> bool {
        match self.children {
            OctTreeChildren::Leaf(_) => true,
            OctTreeChildren::ParentNode(_) => false,
        }
    }
    pub(crate) fn parent(&self) -> Option<&Box<[OctTreeNode<T>; 8]>> {
        match &self.children {
            OctTreeChildren::Leaf(_) => None,
            OctTreeChildren::ParentNode(v) => Some(v),
        }
    }
    pub(crate) fn leaf_value(&self) -> Option<&LeafType<T>> {
        match &self.children {
            OctTreeChildren::Leaf(v) => Some(v),
            OctTreeChildren::ParentNode(_) => None,
        }
    }
    /// return the chunk that the pos is contained in, if the pos is inside of a leaf returns the entire leaf
    /// returns none if pos is out of range
    pub(crate) fn get_chunk(&self, pos: Point3<u32>) -> Option<&OctTreeNode<T>> {
        if pos.x < self.size && pos.y < self.size && pos.z < self.size {
            match &self.children {
                OctTreeChildren::ParentNode(children) => {
                    let get_pos = pos.map(|v| if v >= self.size / 2 { 1u32 } else { 0 });
                    children[get_child_index_size2(get_pos.x, get_pos.y, get_pos.z)].get_chunk(
                        pos.map(|v| {
                            if v >= self.size / 2 {
                                v - self.size / 2
                            } else {
                                v
                            }
                        }),
                    )
                }
                OctTreeChildren::Leaf(_) => Some(self),
            }
        } else {
            None
        }
    }
    /// gets the largest possible homogenous chunk for given pos
    /// returns `None` if pos is out of range
    pub(crate) fn get_homogenous_chunk(&self, pos: Point3<u32>) -> Option<&OctTreeNode<T>> {
        if let Some(chunk) = self.get_chunk(pos) {
            if chunk.is_leaf() {
                Some(chunk)
            } else {
                let child_pos = pos.map(|v| {
                    if v >= (self.size / 2) {
                        v - self.size / 2
                    } else {
                        v
                    }
                });
                let index_pos = pos.map(|v| if v >= (self.size / 2) { 1u32 } else { 0 });

                let children = chunk.parent().unwrap();
                children[get_child_index_size2(index_pos.x, index_pos.y, index_pos.z)]
                    .get_homogenous_chunk(child_pos)
            }
        } else {
            None
        }
    }
}

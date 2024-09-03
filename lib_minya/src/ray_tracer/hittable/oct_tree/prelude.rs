use super::{get_child_index_size2, LeafType, Leafable, OctTreeChildren, OctTreeNode, RayScalar};
use cgmath::{Point3, Vector3};
pub fn distance(a: Vector3<RayScalar>, b: Vector3<RayScalar>) -> RayScalar {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}
// from https://gdbooks.gitbooks.io/3dcollisions/content/Chapter2/static_aabb_aabb.html
pub fn aabb_intersect(a_min: [i32; 3], a_max: [i32; 3], b_min: [i32; 3], b_max: [i32; 3]) -> bool {
    (a_min[0] <= b_max[0] && a_max[0] >= b_min[0])
        && (a_min[1] <= b_max[1] && a_max[1] >= b_min[1])
        && (a_min[2] <= b_max[2] && a_max[2] >= b_min[2])
}

pub(crate) fn get_children_offsets() -> [Point3<u32>; 8] {
    [
        Point3::new(0, 0, 0),
        Point3::new(0, 0, 1),
        Point3::new(0, 1, 0),
        Point3::new(0, 1, 1),
        Point3::new(1, 0, 0),
        Point3::new(1, 0, 1),
        Point3::new(1, 1, 0),
        Point3::new(1, 1, 1),
    ]
}
/// Gets the next powr of 2 that is closest to the given value, if the value is already a power of 2 it returns the same value
/// using https://stackoverflow.com/questions/1322510/given-an-integer-how-do-i-find-the-next-largest-power-of-two-using-bit-twiddlin/1322548#1322548
pub(crate) fn get_next_power(v: u32) -> u32 {
    let mut v1 = v - 1;
    v1 |= v1 >> 1;
    v1 |= v1 >> 2;
    v1 |= v1 >> 4;
    v1 |= v1 >> 8;
    v1 |= v1 >> 16;
    v1 + 1
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
    pub(crate) fn get_chunk(&self, pos: Point3<u32>) -> Option<&OctTreeNode<T>> {
        if pos.x < self.size && pos.y < self.size && pos.z < self.size {
            if pos == Point3::new(0, 0, 0) {
                Some(self)
            } else {
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
            }
        } else {
            None
        }
    }
    /// gets the largest possible homogenous chunk for given pos
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
impl<T: Leafable> LeafType<T> {
    pub(crate) fn try_solid(&self) -> Option<&T> {
        match self {
            Self::Solid(leaf) => Some(leaf),
            Self::Empty => None,
        }
    }
}

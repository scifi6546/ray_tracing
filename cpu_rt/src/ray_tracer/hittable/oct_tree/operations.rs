use super::{HitType, Leafable, OctTree, OctTreeChildren, OctTreeNode};

use std::cmp::PartialEq;
mod combine;
mod load;
mod set;
impl<T: Leafable> OctTree<T> {
    pub fn is_optimal(&self, debug_print: bool) -> bool {
        self.root_node.is_optimal(debug_print)
    }

    // gets offsets of children
}
impl<T: Leafable + PartialEq> OctTree<T> {
    pub(crate) fn get_debug_string(&self) -> String {
        fn get_debug_node<T: Leafable>(node: &OctTreeNode<T>, recurse_level: usize) -> String {
            let recurse_spaces = (0..recurse_level)
                .map(|_| " ")
                .fold(String::new(), |acc, x| acc + x);
            match &node.children {
                OctTreeChildren::Leaf(v) => match v.hit_type() {
                    HitType::Solid => recurse_spaces + &format!("solid leaf, size: {}", node.size),
                    HitType::Volume => {
                        recurse_spaces + &format!("volume leaf, size: {}", node.size)
                    }
                    HitType::Empty => recurse_spaces + &format!("air leaf, size: {}", node.size),
                },

                OctTreeChildren::ParentNode(children) => {
                    recurse_spaces
                        + &format!("parent, size: {}", node.size)
                        + &children
                            .iter()
                            .map(|c| get_debug_node(c, recurse_level + 1))
                            .fold(String::new(), |acc, x| acc + "\n" + &x)
                }
            }
        }
        get_debug_node(&self.root_node, 0)
    }
}
impl<T: Leafable + PartialEq> PartialEq for OctTreeNode<T> {
    fn eq(&self, other: &Self) -> bool {
        if other.size == self.size {
            match &self.children {
                OctTreeChildren::Leaf(self_leaf_value) => match &other.children {
                    OctTreeChildren::Leaf(other_leaf_value) => self_leaf_value == other_leaf_value,
                    OctTreeChildren::ParentNode(_) => false,
                },
                OctTreeChildren::ParentNode(self_children) => match &other.children {
                    OctTreeChildren::Leaf(_) => false,
                    OctTreeChildren::ParentNode(other_children) => {
                        self_children[0] == other_children[0]
                            && self_children[1] == other_children[1]
                            && self_children[2] == other_children[2]
                            && self_children[3] == other_children[3]
                            && self_children[4] == other_children[4]
                            && self_children[5] == other_children[5]
                            && self_children[6] == other_children[6]
                            && self_children[7] == other_children[7]
                    }
                },
            }
        } else {
            false
        }
    }
}
impl<T: Leafable> std::fmt::Debug for OctTree<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.debug_struct("OctTreee")
            .field("children", &self.get_debug_string())
            .finish()
    }
}

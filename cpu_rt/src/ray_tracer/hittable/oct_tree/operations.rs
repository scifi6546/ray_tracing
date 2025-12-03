use super::prelude::*;
use super::{
    prelude::{aabb_intersect, get_children_offsets, get_next_power},
    LeafType, Leafable, OctTree, OctTreeChildren, OctTreeNode,
};
use cgmath::Point3;
use log::info;
use std::cmp::{max, PartialEq};
mod combine;
impl<T: Leafable> OctTree<T> {
    pub fn is_optimal(&self, debug_print: bool) -> bool {
        self.root_node.is_optimal(debug_print)
    }

    // gets offsets of children
}
impl<T: Leafable + PartialEq> OctTree<T> {
    pub fn update(&mut self, position: Point3<u32>, value: T) {
        fn update_node<T: Leafable>(node: &mut OctTreeNode<T>, position: Point3<u32>, value: T) {
            match &mut node.children {
                OctTreeChildren::Leaf(leaf_value) => {
                    if node.size == 1 {
                        if position.x != 0 && position.y != 0 && position.z != 0 {
                            panic!("invalid position: {:?}", position)
                        }
                        *leaf_value = LeafType::Solid(value);
                    } else {
                        let is_same = match leaf_value {
                            LeafType::Solid(old_value) => *old_value == value,
                            LeafType::Empty => false,
                        };
                        if is_same {
                            info!("leaf same, skipping");
                            return;
                        }
                        let idx_position =
                            position.map(|val| if val >= (node.size / 2) { 1u32 } else { 0 });
                        let oct_tree_index =
                            get_child_index_size2(idx_position.x, idx_position.y, idx_position.z);
                        let sub_position = Point3::new(
                            position.x - idx_position.x * (node.size / 2),
                            position.y - idx_position.y * (node.size / 2),
                            position.z - idx_position.z * (node.size / 2),
                        );

                        let new_node = OctTreeNode {
                            children: OctTreeChildren::Leaf(leaf_value.clone()),
                            size: node.size / 2,
                        };

                        let mut new_array = [
                            new_node.clone(),
                            new_node.clone(),
                            new_node.clone(),
                            new_node.clone(),
                            new_node.clone(),
                            new_node.clone(),
                            new_node.clone(),
                            new_node.clone(),
                        ];
                        update_node(&mut new_array[oct_tree_index], sub_position, value);

                        *node = OctTreeNode {
                            children: OctTreeChildren::ParentNode(Box::new(new_array)),
                            size: node.size,
                        };
                    }
                }
                OctTreeChildren::ParentNode(children) => {
                    // getting the index in the children array
                    let idx_position =
                        position.map(|val| if val >= (node.size / 2) { 1u32 } else { 0 });
                    let oct_tree_index =
                        get_child_index_size2(idx_position.x, idx_position.y, idx_position.z);
                    let sub_position = Point3::new(
                        position.x - idx_position.x * (node.size / 2),
                        position.y - idx_position.y * (node.size / 2),
                        position.z - idx_position.z * (node.size / 2),
                    );

                    update_node(&mut children[oct_tree_index], sub_position, value);
                    // checking if the node can be simplified
                    let are_all_children_leaf = children[0].is_leaf()
                        && children[1].is_leaf()
                        && children[2].is_leaf()
                        && children[3].is_leaf()
                        && children[4].is_leaf()
                        && children[5].is_leaf()
                        && children[6].is_leaf()
                        && children[7].is_leaf();
                    if are_all_children_leaf {
                        if children[0].leaf_value().unwrap() == children[1].leaf_value().unwrap()
                            && children[0].leaf_value().unwrap()
                                == children[2].leaf_value().unwrap()
                            && children[0].leaf_value().unwrap()
                                == children[3].leaf_value().unwrap()
                            && children[0].leaf_value().unwrap()
                                == children[4].leaf_value().unwrap()
                            && children[0].leaf_value().unwrap()
                                == children[5].leaf_value().unwrap()
                            && children[0].leaf_value().unwrap()
                                == children[6].leaf_value().unwrap()
                            && children[0].leaf_value().unwrap()
                                == children[7].leaf_value().unwrap()
                        {
                            node.children =
                                OctTreeChildren::Leaf(children[0].leaf_value().unwrap().clone())
                        }
                    }
                }
            }
        }
        if position.x < self.size && position.y < self.size && position.z < self.size {
            update_node(&mut self.root_node, position, value)
        } else {
            let new_size = self.size * 2;
            let empty_node = OctTreeNode {
                children: OctTreeChildren::Leaf(LeafType::Empty),
                size: self.root_node.size,
            };
            let new_root_node = OctTreeNode {
                children: OctTreeChildren::ParentNode(Box::new([
                    self.root_node.clone(),
                    empty_node.clone(),
                    empty_node.clone(),
                    empty_node.clone(),
                    empty_node.clone(),
                    empty_node.clone(),
                    empty_node.clone(),
                    empty_node.clone(),
                ])),
                size: new_size,
            };
            self.root_node = new_root_node;
            self.size = new_size;
            self.update(position, value);
        }
    }
    pub(crate) fn get_debug_string(&self) -> String {
        fn get_debug_node<T: Leafable>(node: &OctTreeNode<T>, recurse_level: usize) -> String {
            let recurse_spaces = (0..recurse_level)
                .map(|_| " ")
                .fold(String::new(), |acc, x| acc + x);
            match &node.children {
                OctTreeChildren::Leaf(v) => {
                    recurse_spaces
                        + &match v {
                            LeafType::Solid(_) => format!("solid leaf, size: {}", node.size),
                            LeafType::Empty => format!("air leaf, size: {}", node.size),
                        }
                }

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
#[cfg(test)]
mod test {
    use cgmath::Point3;

    use crate::ray_tracer::hittable::OctTree;

    #[test]
    fn update_single() {
        let mut oct_tree = OctTree::<bool>::empty();
        oct_tree.update(Point3::new(0, 0, 0), true);
        assert_eq!(oct_tree.root_node.size, 1);
        let leaf = oct_tree.root_node.leaf_value().expect("should be leaf");
        assert_eq!(*leaf.try_solid().expect("must be solid"), true)
    }
    #[test]
    fn update_eight() {
        let mut oct_tree = OctTree::<bool>::empty();
        oct_tree.update(Point3::new(0, 0, 0), true);
        oct_tree.update(Point3::new(0, 0, 1), true);
        oct_tree.update(Point3::new(0, 1, 0), true);
        oct_tree.update(Point3::new(0, 1, 1), true);

        oct_tree.update(Point3::new(1, 0, 0), true);
        oct_tree.update(Point3::new(1, 0, 1), true);
        oct_tree.update(Point3::new(1, 1, 0), true);
        oct_tree.update(Point3::new(1, 1, 1), true);
        assert_eq!(oct_tree.root_node.size, 2);
        let leaf = oct_tree.root_node.leaf_value().expect("should be leaf");

        assert_eq!(*leaf.try_solid().expect("must be solid"), true)
    }
    #[test]
    fn update_16() {
        let mut oct_tree = OctTree::<bool>::empty();
        for x in 0..4 {
            for y in 0..4 {
                for z in 0..4 {
                    oct_tree.update(Point3::new(x, y, z), true);
                    assert!(oct_tree.is_optimal(true));
                }
            }
        }
        assert_eq!(oct_tree.root_node.size, 4);
        let leaf = oct_tree.root_node.leaf_value().expect("should be leaf");

        assert_eq!(*leaf.try_solid().expect("must be solid"), true)
    }
}

use super::{
    super::prelude::get_child_index_size2, LeafType, Leafable, OctTree, OctTreeChildren,
    OctTreeNode,
};
use cgmath::Point3;
use log::info;
impl<T: Leafable + PartialEq> OctTree<T> {
    pub fn set(&mut self, position: Point3<u32>, value: T) {
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
            self.set(position, value);
        }
    }
}
#[cfg(test)]
mod test {
    use cgmath::Point3;

    use crate::ray_tracer::hittable::OctTree;

    #[test]
    fn update_single() {
        let mut oct_tree = OctTree::<bool>::empty();
        oct_tree.set(Point3::new(0, 0, 0), true);
        assert_eq!(oct_tree.root_node.size, 1);
        let leaf = oct_tree.root_node.leaf_value().expect("should be leaf");
        assert_eq!(*leaf.try_solid().expect("must be solid"), true)
    }
    #[test]
    fn update_eight() {
        let mut oct_tree = OctTree::<bool>::empty();
        oct_tree.set(Point3::new(0, 0, 0), true);
        oct_tree.set(Point3::new(0, 0, 1), true);
        oct_tree.set(Point3::new(0, 1, 0), true);
        oct_tree.set(Point3::new(0, 1, 1), true);

        oct_tree.set(Point3::new(1, 0, 0), true);
        oct_tree.set(Point3::new(1, 0, 1), true);
        oct_tree.set(Point3::new(1, 1, 0), true);
        oct_tree.set(Point3::new(1, 1, 1), true);
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
                    oct_tree.set(Point3::new(x, y, z), true);
                    assert!(oct_tree.is_optimal(true));
                }
            }
        }
        assert_eq!(oct_tree.root_node.size, 4);
        let leaf = oct_tree.root_node.leaf_value().expect("should be leaf");

        assert_eq!(*leaf.try_solid().expect("must be solid"), true)
    }
}

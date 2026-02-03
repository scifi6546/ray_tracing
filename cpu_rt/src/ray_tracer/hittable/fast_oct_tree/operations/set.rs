use super::{FastOctTree, Leafable, Node, NodeData, TreePosition};
impl<T: Leafable> FastOctTree<T> {
    /// sets the value of the item at leaf. Automatically resizes as needed
    pub fn set(&mut self, value: T, position: TreePosition) {
        if let Some(root) = self.arena.get_root() {
            let world_size = root.get_world_size();
            if position.x < world_size && position.y < world_size && position.z < world_size {
                let new_root = root.set_child(value, position, &mut self.arena);
                self.arena.update_root(new_root);
            } else {
                let old_root_size = root.size;
                let old_root = self.arena.insert(root);

                let new_root = Node::<T> {
                    data: NodeData::Parent {
                        children: [
                            old_root,
                            self.arena.insert(Node::empty_size(old_root_size)),
                            self.arena.insert(Node::empty_size(old_root_size)),
                            self.arena.insert(Node::empty_size(old_root_size)),
                            self.arena.insert(Node::empty_size(old_root_size)),
                            self.arena.insert(Node::empty_size(old_root_size)),
                            self.arena.insert(Node::empty_size(old_root_size)),
                            self.arena.insert(Node::empty_size(old_root_size)),
                        ],
                    },
                    size: old_root_size + 1,
                };
                let new_root = new_root.set_child(value, position, &mut self.arena);
                self.arena.update_root(new_root);
            }
        } else {
            self.arena.insert(Node::empty());
            let root = self.arena.get_root().expect("should exist").clone();
            let new_root = root.set_child(value, position, &mut self.arena);
            self.arena.update_root(new_root);
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use cgmath::Point3;
    #[test]
    fn get_and_set_one() {
        let mut t = FastOctTree::<u32>::new();
        let l = 0;
        t.set(l, Point3::new(0, 0, 0));
        assert_eq!(t.get(Point3::new(0, 0, 0)).unwrap(), 0);
    }
    #[test]
    fn overwrite() {
        let mut t = FastOctTree::<u32>::new();
        let l = 0;
        t.set(l, Point3::new(0, 0, 0));
        assert_eq!(t.get(Point3::new(0, 0, 0)).unwrap(), l);
        let l2 = 1;
        t.set(l2, Point3::new(0, 0, 0));
        assert_eq!(t.get(Point3::new(0, 0, 0)).unwrap(), l2);
    }
    #[test]
    fn get_and_set_8() {
        let mut t = FastOctTree::<u32>::new();
        let mut i = 0;

        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    println!("setting <{}, {}, {}>", x, y, z);
                    t.set(i, Point3::new(x, y, z));
                    i += 1;
                }
            }
        }
        println!("{:#?}", t);
        let mut i = 0;
        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    println!("getting <{}, {}, {}>", x, y, z);
                    assert_eq!(t.get(Point3::new(x, y, z)).unwrap(), i);
                    i += 1;
                }
            }
        }
    }
}

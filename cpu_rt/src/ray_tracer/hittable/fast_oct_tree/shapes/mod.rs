use super::{prelude::get_next_2_power, ArenaIndex, Node, NodeData};

use super::{super::super::prelude::RayScalar, FastOctTree, Leafable, Voxel};
use cgmath::{MetricSpace, Point3};
impl<T: Leafable> FastOctTree<T> {
    pub(crate) fn sphere(radius: u32, hit_value: T) -> Self {
        let mut tree = FastOctTree::<T>::new();
        let center = Point3::new(
            radius as RayScalar + 0.0,
            radius as RayScalar + 0.0,
            radius as RayScalar + 0.0,
        );
        for x in 0..(2 * radius + 2) {
            for y in 0..(2 * radius + 2) {
                for z in 0..(2 * radius + 2) {
                    let check_point = Point3::new(
                        x as RayScalar + 0.5,
                        y as RayScalar + 0.5,
                        z as RayScalar + 0.5,
                    );
                    let check_point = Point3::new(
                        x as RayScalar + 0.5,
                        y as RayScalar + 0.5,
                        z as RayScalar + 0.5,
                    );
                    if check_point.distance(center) <= radius as RayScalar {
                        tree.set(hit_value.clone(), Point3::new(x, y, z));
                    }
                }
            }
        }
        tree
    }
    fn handle_leaf(
        &mut self,
        size: u32,
        radius: u32,
        center: u32,
        corner: [u32; 3],
        hit_val: T,
    ) -> ArenaIndex {
        todo!()
    }
    pub fn spherev1(radius: u32, hit_value: T) -> Self {
        let size = 2 * get_next_2_power(radius);
        let center = size / 2;
        if size >= 1 {
            let mut tree = Self::new();
            let world_size = 1 << size;
            let children = [
                tree.handle_leaf(size - 1, radius, center, [0, 0, 0], hit_value.clone()),
                tree.handle_leaf(
                    size - 1,
                    radius,
                    center,
                    [0, 0, world_size / 2],
                    hit_value.clone(),
                ),
                tree.handle_leaf(
                    size - 1,
                    radius,
                    center,
                    [0, world_size / 2, 0],
                    hit_value.clone(),
                ),
                tree.handle_leaf(
                    size - 1,
                    radius,
                    center,
                    [0, world_size / 2, world_size / 2],
                    hit_value.clone(),
                ),
                tree.handle_leaf(
                    size - 1,
                    radius,
                    center,
                    [world_size / 2, 0, 0],
                    hit_value.clone(),
                ),
                tree.handle_leaf(
                    size - 1,
                    radius,
                    center,
                    [world_size / 2, 0, world_size / 2],
                    hit_value.clone(),
                ),
                tree.handle_leaf(
                    size - 1,
                    radius,
                    center,
                    [world_size / 2, world_size / 2, 0],
                    hit_value.clone(),
                ),
                tree.handle_leaf(
                    size - 1,
                    radius,
                    center,
                    [world_size / 2, world_size / 2, world_size / 2],
                    hit_value,
                ),
            ];
            tree.arena.update_root(Node {
                data: NodeData::Parent { children },
                size,
            });
            tree
        } else {
            let mut tree = Self::new();
            tree.set(hit_value, Point3::new(0, 0, 0));
            tree
        }
    }
}

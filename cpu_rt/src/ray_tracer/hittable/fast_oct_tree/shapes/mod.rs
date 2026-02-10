use super::{super::super::prelude::RayScalar, FastOctTree, Leafable};
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
                    if check_point.distance(center) <= radius as RayScalar {
                        tree.set(hit_value.clone(), Point3::new(x, y, z));
                    }
                }
            }
        }
        tree
    }
}

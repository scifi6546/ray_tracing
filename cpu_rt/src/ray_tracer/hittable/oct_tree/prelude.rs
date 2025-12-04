use super::RayScalar;
use crate::prelude::Ray;
use cgmath::{Point3, Vector3};

impl Ray {
    pub fn intersect_axis(&self, axis: usize, at: RayScalar) -> RayScalar {
        (at - self.origin[axis]) / self.direction[axis]
    }
    /// gets the time at which the item intersects the x plane
    pub fn intersect_x(&self, at: RayScalar) -> RayScalar {
        self.intersect_axis(0, at)
    }
    /// gets the time at which the item intersects the y plane
    pub fn intersect_y(&self, at: RayScalar) -> RayScalar {
        self.intersect_axis(1, at)
    }
    /// gets the time at which the item intersects the z plane
    pub fn intersect_z(&self, at: RayScalar) -> RayScalar {
        self.intersect_axis(2, at)
    }
    pub fn distance(&self, point: Vector3<RayScalar>) -> RayScalar {
        distance(
            Vector3::new(self.origin.x, self.origin.y, self.origin.z),
            point,
        )
    }
}
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
pub(crate) fn get_child_index_size2(x: u32, y: u32, z: u32) -> usize {
    assert!(x < 2);
    assert!(y < 2);
    assert!(z < 2);
    x as usize * 4 + y as usize * 2 + z as usize
}

use crate::prelude::{Ray, RayScalar};
use cgmath::{prelude::*, Point3};
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
    pub fn distance(&self, point: Point3<RayScalar>) -> RayScalar {
        self.origin.distance(point)
    }
}

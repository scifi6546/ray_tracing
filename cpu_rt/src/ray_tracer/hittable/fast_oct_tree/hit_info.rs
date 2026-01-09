use super::Leafable;
use crate::prelude::RayScalar;
use cgmath::{Point3, Vector3};
pub enum HitInfo<T: Leafable> {
    Solid {
        hit_value: T::Material,
        hit_position: Point3<RayScalar>,
        normal: Vector3<RayScalar>,
    },
    Volume {
        hit_value: T::Material,
        hit_position: Point3<RayScalar>,
    },
}

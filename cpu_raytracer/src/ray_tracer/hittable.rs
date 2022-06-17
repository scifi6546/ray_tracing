mod constant_medium;
mod flip_normals;
mod rect;
mod render_box;
mod rotation;
mod sphere;
mod translate;

use super::{Aabb, Material, Ray};

use cgmath::{InnerSpace, Point2, Point3, Vector3};

pub use constant_medium::ConstantMedium;
pub use flip_normals::FlipNormals;
pub use rect::{XYRect, XZRect, YZRect};
pub use render_box::RenderBox;
pub use rotation::RotateY;
pub use sphere::{MovingSphere, Sphere};

use std::{cell::RefCell, rc::Rc};
pub use translate::Translate;

pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<Aabb>;
    /// probability of hitting the box for given ray going towards point
    fn prob(&self, ray: Ray) -> f32;
    fn generate_ray_in_area(&self, origin: Point3<f32>, time: f32) -> RayAreaInfo;
}
#[derive(Clone)]
pub struct HitRecord {
    pub position: Point3<f32>,
    pub normal: Vector3<f32>,
    pub t: f32,
    pub front_face: bool,
    pub uv: Point2<f32>,
    pub material: Rc<RefCell<dyn Material>>,
}

impl HitRecord {
    pub fn new(
        ray: &Ray,
        position: Point3<f32>,
        normal: Vector3<f32>,
        t: f32,
        uv: Point2<f32>,
        material: Rc<RefCell<dyn Material>>,
    ) -> Self {
        let front_face = ray.direction.dot(normal) < 0.0;
        Self {
            position,
            normal: if front_face { normal } else { -1.0 * normal },
            t,
            front_face,
            uv,
            material,
        }
    }
}
pub struct RayAreaInfo {
    pub to_area: Ray,
    pub area: f32,
    pub direction: Vector3<f32>,
    pub normal: Vector3<f32>,
}
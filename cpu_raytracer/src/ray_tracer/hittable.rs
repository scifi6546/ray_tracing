mod constant_medium;
mod flip_normals;
mod rect;
mod render_box;
mod rotation;
mod sphere;
mod translate;

use super::{Aabb, Material, Ray};

use cgmath::{InnerSpace, Matrix4, Point2, Point3, SquareMatrix, Vector3, Vector4};

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
#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub world_transform: Matrix4<f32>,
}
impl Transform {
    pub fn get_inverse(&self) -> Transform {
        Self {
            world_transform: self
                .world_transform
                .invert()
                .expect("transform is not invertible"),
        }
    }
    fn from_matrix(world_transform: Matrix4<f32>) -> Self {
        Self { world_transform }
    }
    pub fn identity() -> Self {
        Self::from_matrix(Matrix4::identity())
    }
    pub fn translation(translation: Vector3<f32>) -> Self {
        Self::from_matrix(Matrix4::from_translation(translation))
    }
    fn mul_ray(&self, ray: Ray) -> Ray {
        let direction_world = ray.origin + ray.direction;
        let direction_end = self.world_transform * direction_world.to_homogeneous();
        let world_origin: Vector4<f32> = self.world_transform * ray.origin.to_homogeneous();

        let direction_world = direction_end - world_origin;
        let direction = Vector3::new(direction_world.x, direction_world.y, direction_world.z);
        Ray {
            origin: Point3::from_homogeneous(world_origin),
            direction,
            time: ray.time,
        }
    }
    fn mul_point3(&self, point: Point3<f32>) -> Point3<f32> {
        Point3::from_homogeneous(self.world_transform * point.to_homogeneous())
    }
}
impl std::ops::Mul<&Point3<f32>> for &Transform {
    type Output = Point3<f32>;

    fn mul(self, rhs: &Point3<f32>) -> Self::Output {
        self.mul_point3(*rhs)
    }
}
impl std::ops::Mul<&Point3<f32>> for Transform {
    type Output = Point3<f32>;

    fn mul(self, rhs: &Point3<f32>) -> Self::Output {
        (&self).mul_point3(*rhs)
    }
}
impl std::ops::Mul<Point3<f32>> for Transform {
    type Output = Point3<f32>;

    fn mul(self, rhs: Point3<f32>) -> Self::Output {
        (&self).mul_point3(rhs)
    }
}
impl std::ops::Mul<&Ray> for &Transform {
    type Output = Ray;

    fn mul(self, rhs: &Ray) -> Self::Output {
        self.mul_ray(*rhs)
    }
}
impl std::ops::Mul<Ray> for &Transform {
    type Output = Ray;

    fn mul(self, rhs: Ray) -> Self::Output {
        self.mul_ray(rhs)
    }
}
impl std::ops::Mul<Ray> for Transform {
    type Output = Ray;

    fn mul(self, rhs: Ray) -> Self::Output {
        (&self).mul_ray(rhs)
    }
}
#[derive(Clone)]
pub struct Object {
    pub shape: Rc<dyn Hittable>,
    pub transform: Transform,
}
impl Object {
    pub fn new(shape: Rc<dyn Hittable>, transform: Transform) -> Self {
        Self { shape, transform }
    }
}
impl Hittable for Object {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let ray = &self.transform * ray;
        if let Some(hit) = self.shape.hit(&ray, t_min, t_max) {
            let inv = self.transform.get_inverse();
            let world_position = &inv * &hit.position;

            let hit_normal_end_world = inv * (hit.position + hit.normal);
            let normal_world = hit_normal_end_world - world_position;
            Some(HitRecord {
                position: world_position,
                normal: normal_world,
                t: hit.t,
                front_face: hit.front_face,
                uv: hit.uv,
                material: hit.material,
            })
        } else {
            None
        }
    }

    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<Aabb> {
        if let Some(aabb) = self.shape.bounding_box(time_0, time_1) {
            let inv = self.transform.get_inverse();
            let corner_a = inv * aabb.minimum;
            let corner_b = inv * aabb.maximum;
            let min_x = if corner_a.x < corner_b.x {
                corner_a.x
            } else {
                corner_b.x
            };
            let min_y = if corner_a.y < corner_b.y {
                corner_a.y
            } else {
                corner_b.y
            };
            let min_z = if corner_a.z < corner_b.z {
                corner_a.z
            } else {
                corner_b.z
            };

            let max_x = if corner_a.x > corner_b.x {
                corner_a.x
            } else {
                corner_b.x
            };
            let max_y = if corner_a.y > corner_b.y {
                corner_a.y
            } else {
                corner_b.y
            };
            let max_z = if corner_a.z > corner_b.z {
                corner_a.z
            } else {
                corner_b.z
            };
            Some(Aabb {
                minimum: Point3::new(min_x, min_y, min_z),
                maximum: Point3::new(max_x, max_y, max_z),
            })
        } else {
            None
        }
    }

    fn prob(&self, ray: Ray) -> f32 {
        let inv = self.transform.get_inverse();
        self.shape.prob(inv * ray)
    }

    fn generate_ray_in_area(&self, origin: Point3<f32>, time: f32) -> RayAreaInfo {
        let out_area_info = self
            .shape
            .generate_ray_in_area(self.transform * origin, time);
        let inv = self.transform.get_inverse();
        let to_area = inv * out_area_info.to_area;
        let scaling = inv * Point3::new(1.0f32, 0.0, 0.0) - inv * Point3::new(0.0f32, 0.0, 0.0);
        let scaling = scaling.magnitude().abs();
        let area = out_area_info.area * scaling;

        let direction_end = inv * (out_area_info.to_area.origin + out_area_info.direction);
        let direction = direction_end - to_area.origin;
        let normal_end = inv * (out_area_info.to_area.origin + out_area_info.normal);
        let normal = normal_end - to_area.origin;
        RayAreaInfo {
            to_area,
            area,
            direction,
            normal,
        }
    }
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

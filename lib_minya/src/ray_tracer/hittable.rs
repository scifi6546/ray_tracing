mod constant_medium;
mod oct_tree;
mod rect;
mod render_box;
mod sphere;
pub mod voxel_world;

use super::{Aabb, Material, Ray};

use cgmath::{InnerSpace, Matrix3, Matrix4, Point2, Point3, SquareMatrix, Vector3, Vector4};

use crate::ray_tracer::hittable::MaterialEffect::NoEmmit;
use crate::ray_tracer::pdf::ScatterRecord;
use base_lib::RgbColor;
pub use constant_medium::ConstantMedium;
use dyn_clone::{clone_box, DynClone};
pub use oct_tree::{OctTree, VoxelMaterial};
pub use rect::{XYRect, XZRect, YZRect};
pub use render_box::RenderBox;
pub use sphere::{MovingSphere, Sphere};
use std::ops::Deref;
pub use voxel_world::VoxelWorld;
///Objects that can be hit
pub mod hittable_objects {
    pub use super::rect::{XYRect, XZRect, YZRect};
    pub(crate) use super::voxel_world::VoxelMap;
    pub use super::voxel_world::{CubeMaterial, VoxelModel, VoxelWorld};
}
pub trait Hittable: Send + Sync + DynClone {
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
    pub fn translate(self, translation: Vector3<f32>) -> Self {
        self * Self::from_matrix(Matrix4::from_translation(-1.0 * translation))
    }
    pub fn rotate_x(self, rotation_deg: f32) -> Self {
        self * Self::from_matrix(cgmath::Matrix4::from_angle_x(cgmath::Deg(rotation_deg)))
    }
    pub fn rotate_y(self, rotation_deg: f32) -> Self {
        self * Self::from_matrix(cgmath::Matrix4::from_angle_y(cgmath::Deg(rotation_deg)))
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

    fn mul_vec3(&self, vec: Vector3<f32>) -> Vector3<f32> {
        let world_vec = Vector4::new(vec.x, vec.y, vec.z, 1.0);
        let output = self.mul_vec4(world_vec);
        Vector3::new(output.x, output.y, output.z)
    }
    fn mul_vec4(&self, vec: Vector4<f32>) -> Vector4<f32> {
        self.world_transform * vec
    }
    fn mul_self(&self, rhs: Self) -> Self {
        Self {
            world_transform: self.world_transform * rhs.world_transform,
        }
    }
}
impl std::ops::Mul<Transform> for Transform {
    type Output = Self;
    fn mul(self, rhs: Transform) -> Self::Output {
        (&self).mul_self(rhs)
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
impl std::ops::Mul<Vector3<f32>> for Transform {
    type Output = Vector3<f32>;
    fn mul(self, rhs: Vector3<f32>) -> Self::Output {
        (&self).mul_vec3(rhs)
    }
}
impl std::ops::Mul<Vector4<f32>> for Transform {
    type Output = Vector4<f32>;
    fn mul(self, rhs: Vector4<f32>) -> Self::Output {
        (&self).mul_vec4(rhs)
    }
}

pub struct Object {
    pub shape: Box<dyn Hittable + Send>,
    pub transform: Transform,
}
impl Clone for Object {
    fn clone(&self) -> Self {
        Self {
            shape: clone_box(self.shape.deref()),
            transform: self.transform,
        }
    }
}

impl Object {
    pub fn new(shape: Box<dyn Hittable + Send>, transform: Transform) -> Self {
        Self { shape, transform }
    }
}

impl Hittable for Object {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let shape_ray = &self.transform * ray;
        fn get_three(m: &Matrix4<f32>) -> Matrix3<f32> {
            let v1 = m[0];
            let v2 = m[1];
            let v3 = m[2];
            let t = [v1.x, v1.y, v1.z, v2.x, v2.y, v2.z, v3.x, v3.y, v3.z];
            let m: &Matrix3<f32> = (&t).into();
            return m.clone();
        }
        if let Some(hit) = self.shape.hit(&shape_ray, t_min, t_max) {
            let three = get_three(&self.transform.world_transform);
            let three_inv = three.invert().unwrap();
            let inv = self.transform.get_inverse();
            let world_position = inv * hit.position;

            let normal_world = three_inv * hit.normal;
            //let normal_world = hit.normal;
            let front_face = ray.direction.dot(normal_world) <= 0.0;
            //let normal_world = inv * Vector4::new(hit.normal.x, hit.normal.y, hit.normal.z, 0.0);
            //let normal_world = Vector3::new(normal_world.x, normal_world.y, normal_world.z);
            //let normal_world = hit.normal;
            let h = Some(HitRecord {
                position: world_position,
                normal: normal_world,
                t: hit.t,
                front_face,
                uv: hit.uv,
                material_effect: hit.material_effect,
            });

            h
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
        let end_point = inv * out_area_info.end_point;
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
            end_point,
        }
    }
}
#[derive(Clone, Debug)]
pub struct HitRay {
    position: Point3<f32>,
    direction: Vector3<f32>,
    normal: Vector3<f32>,

    front_face: bool,
    uv: Point2<f32>,
}
impl HitRay {
    pub(crate) fn position(&self) -> Point3<f32> {
        self.position
    }
    pub(crate) fn direction(&self) -> Vector3<f32> {
        self.direction
    }
    pub(crate) fn normal(&self) -> Vector3<f32> {
        self.normal
    }

    pub(crate) fn front_face(&self) -> bool {
        self.front_face
    }
    pub(crate) fn uv(&self) -> Point2<f32> {
        self.uv
    }
}
#[derive(Clone, Debug)]
pub struct HitRecord {
    pub position: Point3<f32>,
    pub normal: Vector3<f32>,
    pub t: f32,
    pub front_face: bool,
    pub uv: Point2<f32>,
    pub material_effect: MaterialEffect,
}
#[derive(Clone, Debug)]
pub enum MaterialEffect {
    Scatter(ScatterRecord),
    Emmit(RgbColor),
    NoEmmit,
}

impl HitRecord {
    pub fn new_ref<M: Material>(
        ray: &Ray,
        position: Point3<f32>,
        normal: Vector3<f32>,
        t: f32,
        uv: Point2<f32>,
        material: &M,
    ) -> Self {
        let front_face = ray.direction.dot(normal) <= 0.0;

        let hit_ray = HitRay {
            position,
            direction: ray.direction,
            //normal: if front_face { normal } else { -normal },
            normal,

            front_face,
            uv,
        };
        let emit_option = material.emmit(&hit_ray);
        let material_effect = if emit_option.is_some() {
            MaterialEffect::Emmit(emit_option.unwrap())
        } else if let Some(scatter) = material.scatter(ray.clone(), &hit_ray) {
            MaterialEffect::Scatter(scatter)
        } else {
            NoEmmit
        };
        Self {
            position,
            normal: hit_ray.normal,
            t,
            front_face,
            uv,
            material_effect,
        }
    }
    pub fn new(
        ray: &Ray,
        position: Point3<f32>,
        normal: Vector3<f32>,
        t: f32,
        uv: Point2<f32>,
        material: &dyn Material,
    ) -> Self {
        let front_face = ray.direction.dot(normal) <= 0.0;

        let hit_ray = HitRay {
            position,
            direction: ray.direction,
            normal,
            front_face,
            uv,
        };
        let emit_option = material.emmit(&hit_ray);
        let material_effect = if emit_option.is_some() {
            MaterialEffect::Emmit(emit_option.unwrap())
        } else if let Some(scatter) = material.scatter(ray.clone(), &hit_ray) {
            MaterialEffect::Scatter(scatter)
        } else {
            NoEmmit
        };
        Self {
            position,
            normal: hit_ray.normal,
            t,
            front_face,
            uv,
            material_effect,
        }
    }
}
pub struct RayAreaInfo {
    pub to_area: Ray,
    pub area: f32,
    pub direction: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub end_point: Point3<f32>,
}

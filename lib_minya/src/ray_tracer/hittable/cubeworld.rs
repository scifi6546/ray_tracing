use super::{Aabb, FlipNormals, HitRecord, Hittable, Material, XYRect, XZRect, YZRect};
use crate::prelude::*;
use crate::ray_tracer::hittable::RayAreaInfo;
use cgmath::{InnerSpace, MetricSpace, Point2, Point3, Vector2, Vector3};
use dyn_clone::clone_box;
use std::ops::{Deref, Neg};
#[derive(Clone)]
struct Voxels {
    data: Vec<bool>,
    x_dim: usize,
    y_dim: usize,
    z_dim: usize,
}
enum HitResult {
    Hit {
        position: Vector3<f32>,
        normal: Vector3<f32>,
    },
    DidNotHit,
}
fn min_idx_vec(v: Vector3<f32>) -> usize {
    let mut min_val = v.x;
    let mut min_idx = 0;

    if min_val > v.y {
        min_val = v.y;
        min_idx = 1;
    }
    if min_val > v.z {
        min_val = v.z;
        min_idx = 2;
    }
    return min_idx;
}
impl Voxels {
    pub fn new(x_dim: usize, y_dim: usize, z_dim: usize) -> Self {
        Self {
            data: vec![false; x_dim * y_dim * z_dim],
            x_dim,
            y_dim,
            z_dim,
        }
    }
    fn get_idx(&self, x: usize, y: usize, z: usize) -> usize {
        x + y * self.x_dim + z * self.x_dim * self.y_dim
    }
    pub fn in_range(&self, x: isize, y: isize, z: isize) -> bool {
        x >= 0
            && y >= 0
            && z >= 0
            && x < self.x_dim as isize
            && y < self.y_dim as isize
            && z < self.z_dim as isize
    }
    pub fn get(&self, x: usize, y: usize, z: usize) -> bool {
        self.data[self.get_idx(x, y, z)]
    }
    pub fn update(&mut self, x: isize, y: isize, z: isize, val: bool) {
        if self.in_range(x, y, z) {
            let idx = self.get_idx(x as usize, y as usize, z as usize);
            self.data[idx] = val;
        }
    }
    pub fn save_images(&self) {
        for i in 0..self.z_dim {
            let save_img =
                image::RgbaImage::from_fn(self.x_dim as u32, self.y_dim as u32, |x, y| {
                    if self.get(x as usize, y as usize, i) {
                        [255, 255, 255, 255].into()
                    } else {
                        [0, 0, 0, 255].into()
                    }
                });
            save_img
                .save(format!("test/{}.png", i))
                .expect("failed to save layer");
        }
    }
    pub fn trace_voxels(&self, origin: Vector3<f32>, direction: Vector3<f32>) -> HitResult {
        let step_size = 1.0 / direction.map(|e| e.abs());
        let mut step_dir = Vector3::new(0.0, 0.0, 0.0);
        let mut next_dist = Vector3::new(0.0, 0.0, 0.0);
        if direction.x < 0.0 {
            step_dir.x = -1.0;
            next_dist.x = -1.0 * (origin.x.fract()) / direction.x;
        } else {
            step_dir.x = 1.0;
            next_dist.x = (1.0 - origin.x.fract()) / direction.x;
        }

        if direction.y < 0.0 {
            step_dir.y = -1.0;
            next_dist.y = (origin.y.fract().neg()) / direction.y;
        } else {
            step_dir.y = 1.0;
            next_dist.y = (1.0 - origin.y.fract()) / direction.y;
        }
        if direction.z < 0.0 {
            step_dir.z = -1.0;
            next_dist.z = (origin.z.fract().neg()) / direction.z;
        } else {
            step_dir.z = 1.0;
            next_dist.z = (1.0 - origin.z.fract()) / direction.z;
        }

        let mut voxel_pos = origin.map(|e| e.floor());
        let mut current_pos = origin;

        loop {
            let mut normal = Vector3::new(0.0, 0.0, 0.0);
            let min_idx = min_idx_vec(next_dist);
            if min_idx == 0 {
                //min_idx = 0
                voxel_pos.x += step_dir.x;
                current_pos += direction * next_dist.x;
                next_dist = next_dist.map(|f| f - next_dist.x);
                next_dist.x += step_size.x;
                normal = Vector3::new(step_dir.x.neg(), 0.0, 0.0);
            } else if min_idx == 1 {
                //min_idx = 1
                voxel_pos.y += step_dir.y;
                current_pos += direction * next_dist.y;
                next_dist = next_dist.map(|f| f - next_dist.y);
                next_dist.y += step_size.y;
                normal = Vector3::new(0.0, step_dir.y.neg(), 0.0);
            } else {
                //min_idx = 2
                voxel_pos.z += step_dir.z;
                current_pos += direction * next_dist.z;
                next_dist = next_dist.map(|f| f - next_dist.z);
                next_dist.z += step_size.z;
                normal = Vector3::new(0.0, 0.0, step_dir.z.neg());
            }
            let x_pos = voxel_pos.x as isize;
            let y_pos = voxel_pos.y as isize;
            let z_pos = voxel_pos.z as isize;
            if self.in_range(x_pos, y_pos, z_pos) {
                let r = self.get(x_pos as usize, y_pos as usize, z_pos as usize);
                if r {
                    return HitResult::Hit {
                        position: voxel_pos,
                        normal,
                    };
                } else {
                    continue;
                }
            } else {
                return HitResult::DidNotHit;
            }
        }
    }
}
#[derive(Clone)]
struct CheckRes {
    direction: Vector3<f32>,
    origin: Vector3<f32>,
    t: f32,
}
pub struct CubeWorld {
    material: Box<dyn Material>,
    voxels: Voxels,
    x: i32,
    y: i32,
    z: i32,
}
impl CubeWorld {
    const OFFSET: f32 = 0.1;
    pub fn new(material: Box<dyn Material>, x: i32, y: i32, z: i32) -> Self {
        let mut voxels = Voxels::new(x as usize, y as usize, z as usize);
        let center = Vector3::new(x as f32 / 2.0, y as f32 / 2.0, z as f32 / 2.0);
        let radius = 3.0;
        for i in 0..x as isize {
            for j in 0..y as isize {
                for k in 0..z as isize {
                    let pos = Vector3::new(i as f32, j as f32, k as f32);
                    let p_val = (pos - center).magnitude() < radius;

                    voxels.update(i, j, k, p_val);
                }
            }
        }
        voxels.update(0, 0, 0, true);
        voxels.update(0, 1, 0, true);
        voxels.update(5, 5, 5, true);
        voxels.save_images();
        Self {
            material,
            voxels,
            x,
            y,
            z,
        }
    }
    fn check_x(
        &self,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
        x: f32,
        normal: Vector3<f32>,
    ) -> Option<CheckRes> {
        let t = (x - ray.origin.x) / ray.direction.x;
        if t > t_min && t < t_max {
            let pos = ray.at(t) - Self::OFFSET * ray.direction.normalize();

            if pos.y > 0.0 && pos.y < self.y as f32 && pos.z > 0.0 && pos.z < self.z as f32 {
                Some(CheckRes {
                    direction: ray.direction,
                    origin: Vector3::new(pos.x, pos.y, pos.z),
                    t,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
    fn check_y(
        &self,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
        y: f32,
        normal: Vector3<f32>,
    ) -> Option<CheckRes> {
        let t = (y - ray.origin.y) / ray.direction.y;
        if t > t_min && t < t_max {
            let pos = ray.at(t) - Self::OFFSET * ray.direction.normalize();

            if pos.x > 0.0 && pos.x < self.x as f32 && pos.z > 0.0 && pos.z < self.z as f32 {
                Some(CheckRes {
                    direction: ray.direction,
                    origin: Vector3::new(pos.x, pos.y, pos.z),
                    t,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
    fn check_z(
        &self,
        ray: &Ray,
        t_min: f32,
        t_max: f32,
        z: f32,
        normal: Vector3<f32>,
    ) -> Option<CheckRes> {
        let t = (z - ray.origin.z) / ray.direction.z;
        if t > t_min && t < t_max {
            let pos = ray.at(t) - Self::OFFSET * ray.direction.normalize();

            if pos.x > 0.0 && pos.x < self.x as f32 && pos.y > 0.0 && pos.y < self.y as f32 {
                Some(CheckRes {
                    direction: ray.direction,
                    origin: Vector3::new(pos.x, pos.y, pos.z),
                    t,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
    fn manage_hit_res(
        &self,
        ray: &Ray,
        hit: HitResult,
        t_min: f32,
        t_max: f32,
    ) -> Option<HitRecord> {
        match hit {
            HitResult::Hit { position, normal } => {
                let dist = ray.origin - position;
                let t =
                    Vector3::new(dist.x, dist.y, dist.z).magnitude() / ray.direction.magnitude();
                if t > t_min && t < t_max {
                    Some(HitRecord::new(
                        ray,
                        Point3::new(position.x, position.y, position.z),
                        normal,
                        t,
                        Point2::new(0.0, 0.0),
                        clone_box(self.material.as_ref()),
                    ))
                } else {
                    None
                }
            }
            HitResult::DidNotHit => None,
        }
    }
}
impl Clone for CubeWorld {
    fn clone(&self) -> Self {
        Self {
            material: clone_box(self.material.deref()),
            voxels: self.voxels.clone(),
            x: self.x,
            y: self.y,
            z: self.y,
        }
    }
}
impl Hittable for CubeWorld {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let aabb = self.bounding_box(t_min, t_max).expect("failed to get aabb");
        if aabb.contains_point(ray.origin) {
            let hit_res = self.voxels.trace_voxels(
                Vector3::new(ray.origin.x, ray.origin.y, ray.origin.z),
                ray.direction,
            );
            return self.manage_hit_res(ray, hit_res, t_min, t_max);
        }
        let solutions = [
            self.check_x(ray, t_min, t_max, 0.0, Vector3::new(-1.0, 0.0, 0.0)),
            self.check_x(
                ray,
                t_min,
                t_max,
                self.x as f32,
                Vector3::new(1.0, 0.0, 0.0),
            ),
            self.check_y(ray, t_min, t_max, 0.0, Vector3::new(0.0, -1.0, 0.0)),
            self.check_y(
                ray,
                t_min,
                t_max,
                self.y as f32,
                Vector3::new(0.0, 1.0, 0.0),
            ),
            self.check_z(ray, t_min, t_max, 0.0, Vector3::new(0.0, 0.0, -1.0)),
            self.check_z(
                ray,
                t_min,
                t_max,
                self.z as f32,
                Vector3::new(0.0, 0.0, 1.0),
            ),
        ];
        let mut min_dist = f32::MAX;

        let mut min_index = usize::MAX;
        for i in 0..solutions.len() {
            let check_opt = solutions[i].clone();
            if check_opt.is_some() {
                let check = check_opt.unwrap();

                let distance = Point3::new(check.origin.x, check.origin.y, check.origin.z)
                    .distance(ray.origin);
                if min_dist > distance {
                    min_index = i;
                    min_dist = distance;
                }
            }
        }
        if min_index != usize::MAX {
            let s = solutions[min_index].clone().unwrap();
            let hit_res = self.voxels.trace_voxels(s.origin, s.direction);
            return self.manage_hit_res(ray, hit_res, t_min, t_max);
        } else {
            return None;
        }
        let x0 = self.check_x(ray, t_min, t_max, 0.0, Vector3::new(-1.0, 0.0, 0.0));
        if x0.is_some() {
            let x0 = x0.unwrap();
            let hit_res = self.voxels.trace_voxels(x0.origin, x0.direction);
            return self.manage_hit_res(ray, hit_res, t_min, t_max);
        }

        let x_max = self.check_x(
            ray,
            t_min,
            t_max,
            self.x as f32,
            Vector3::new(-1.0, 0.0, 0.0),
        );
        if x_max.is_some() {
            let x_max = x_max.unwrap();
            let hit_res = self.voxels.trace_voxels(x_max.origin, x_max.direction);
            return self.manage_hit_res(ray, hit_res, t_min, t_max);
        }

        let y0 = self.check_y(ray, t_min, t_max, 0.0, Vector3::new(0.0, -1.0, 0.0));
        if y0.is_some() {
            let y0 = y0.unwrap();
            let hit_res = self.voxels.trace_voxels(y0.origin, y0.direction);
            return self.manage_hit_res(ray, hit_res, t_min, t_max);
        }

        let y_max = self.check_y(
            ray,
            t_min,
            t_max,
            self.y as f32,
            Vector3::new(0.0, 1.0, 0.0),
        );
        if y_max.is_some() {
            let y_max = y_max.unwrap();
            let hit_res = self.voxels.trace_voxels(y_max.origin, y_max.direction);
            return self.manage_hit_res(ray, hit_res, t_min, t_max);
        }

        let z0 = self.check_z(ray, t_min, t_max, 0.0, Vector3::new(0.0, 0.0, -1.0));
        if z0.is_some() {
            let z0 = z0.unwrap();
            let hit_res = self.voxels.trace_voxels(z0.origin, z0.direction);
            return self.manage_hit_res(ray, hit_res, t_min, t_max);
        }

        let z_max = self.check_z(
            ray,
            t_min,
            t_max,
            self.z as f32,
            Vector3::new(0.0, 0.0, 1.0),
        );
        if z_max.is_some() {
            let z_max = z_max.unwrap();
            let hit_res = self.voxels.trace_voxels(z_max.origin, z_max.direction);
            return self.manage_hit_res(ray, hit_res, t_min, t_max);
        }
        return None;
    }

    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<Aabb> {
        Some(Aabb {
            minimum: Point3::new(0.0, 0.0, 0.0),
            maximum: Point3::new(self.x as f32, self.y as f32, self.z as f32),
        })
    }

    fn prob(&self, ray: Ray) -> f32 {
        todo!()
    }

    fn generate_ray_in_area(&self, origin: Point3<f32>, time: f32) -> RayAreaInfo {
        todo!()
    }
}

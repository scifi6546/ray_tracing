use super::{super::Lambertian, Aabb, HitRecord, Hittable};

use crate::ray_tracer::{
    hittable::{HitRay, RayAreaInfo},
    pdf::ScatterRecord,
    texture::SolidColor,
};
use crate::{prelude::*, ray_tracer::Material};
use cgmath::{prelude::*, Point2, Point3, Vector3};
pub(crate) use voxel_map::VoxelMap;
mod perlin;
mod voxel_map;
mod voxel_model;

pub use perlin::{PerlinBuilder, PerlinNoise};
use std::ops::Neg;
pub use voxel_model::VoxelModel;
#[derive(Debug)]
enum HitResult<T: Solid + std::fmt::Debug> {
    Hit {
        position: Point3<RayScalar>,
        normal: Vector3<RayScalar>,
        voxel: T,
    },
    DidNotHit,
}
fn min_idx_vec(v: Vector3<RayScalar>) -> usize {
    let mut min_val = v.x;
    let mut min_idx = 0;

    if min_val > v.y {
        min_val = v.y;
        min_idx = 1;
    }
    if min_val > v.z {
        return 2;
    }
    return min_idx;
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum CubeType {
    Solid,
    Translucent { density: RayScalar },
    Air,
}
trait Solid {
    fn solid(&self) -> CubeType;
}
impl Solid for bool {
    fn solid(&self) -> CubeType {
        match self {
            true => CubeType::Solid,
            false => CubeType::Air,
        }
    }
}
#[derive(Clone)]
struct Voxels<T: Clone + Solid> {
    data: Vec<T>,
    x_dim: usize,
    y_dim: usize,
    z_dim: usize,
}
fn step_translucent(
    position: Point3<RayScalar>,
    direction: Vector3<RayScalar>,
    density: RayScalar,
) -> Option<Point3<RayScalar>> {
    assert!(density <= 1.0);
    assert!(density >= 0.0);
    let max_distance = {
        let three: RayScalar = 3.0;
        three.sqrt()
    };

    let max_r = 1.0 / density;
    let r = rand_scalar(0.0, max_r);

    if r <= 1.0 {
        let dist = max_distance * r;
        let next_pos = position + dist * direction.normalize();
        let next_voxel = next_pos.map(|f| f.floor() as i32);
        if next_voxel == position.map(|f| f.floor() as i32) {
            Some(next_pos)
        } else {
            None
        }
    } else {
        None
    }
}
impl<T: Clone + Solid + std::fmt::Debug> Voxels<T> {
    /// gets size of voxel grid
    pub(crate) fn size(&self) -> Vector3<usize> {
        Vector3::new(self.x_dim, self.y_dim, self.z_dim)
    }
    pub fn new(x_dim: usize, y_dim: usize, z_dim: usize, default_value: T) -> Self {
        Self {
            data: vec![default_value; x_dim * y_dim * z_dim],
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
    pub fn get(&self, x: usize, y: usize, z: usize) -> T {
        self.data[self.get_idx(x, y, z)].clone()
    }
    pub fn update(&mut self, x: isize, y: isize, z: isize, val: T) {
        if self.in_range(x, y, z) {
            let idx = self.get_idx(x as usize, y as usize, z as usize);
            self.data[idx] = val;
        } else {
            error!("out of range ({}, {}, {})", x, y, z)
        }
    }

    pub fn trace_voxels(
        &self,
        origin: Point3<RayScalar>,
        direction: Vector3<RayScalar>,
    ) -> HitResult<T> {
        let step_size = 1.0 / direction.map(|e| e.abs());
        let mut step_dir = Vector3::<RayScalar>::zero();
        let mut next_dist = Vector3::zero();
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

        let mut voxel_pos = origin.map(|e| e as isize);
        let mut current_pos = origin;

        loop {
            let min_idx = min_idx_vec(next_dist);
            let normal = if min_idx == 0 {
                //min_idx = 0
                voxel_pos.x += if step_dir.x.is_sign_positive() { 1 } else { -1 };
                current_pos += direction * next_dist.x;
                next_dist = next_dist.map(|f| f - next_dist.x);
                next_dist.x += step_size.x;
                Vector3::new(step_dir.x.neg(), 0.0, 0.0).normalize()
            } else if min_idx == 1 {
                //min_idx = 1
                voxel_pos.y += if step_dir.y.is_sign_positive() { 1 } else { -1 };
                current_pos += direction * next_dist.y;
                next_dist = next_dist.map(|f| f - next_dist.y);
                next_dist.y += step_size.y;
                Vector3::new(0.0, step_dir.y.neg(), 0.0).normalize()
            } else if min_idx == 2 {
                //min_idx = 2
                voxel_pos.z += if step_dir.z.is_sign_positive() { 1 } else { -1 };
                current_pos += direction * next_dist.z;
                next_dist = next_dist.map(|f| f - next_dist.z);
                next_dist.z += step_size.z;
                Vector3::new(0.0, 0.0, step_dir.z.neg()).normalize()
            } else {
                panic!("invalid min_idx")
            };
            let x_pos = voxel_pos.x as isize;
            let y_pos = voxel_pos.y as isize;
            let z_pos = voxel_pos.z as isize;
            if self.in_range(x_pos, y_pos, z_pos) {
                let voxel = self.get(x_pos as usize, y_pos as usize, z_pos as usize);
                match voxel.solid() {
                    CubeType::Translucent { density } => {
                        if let Some(position) =
                            step_translucent(current_pos, direction.normalize(), density)
                        {
                            return HitResult::Hit {
                                position,
                                normal,
                                voxel,
                            };
                        }
                    }
                    CubeType::Solid => {
                        return HitResult::Hit {
                            position: current_pos,
                            normal,
                            voxel,
                        };
                    }
                    CubeType::Air => {}
                }
            } else {
                return HitResult::DidNotHit;
            }
        }
    }
}
#[derive(Clone, Debug)]
struct CheckRes {
    direction: Vector3<RayScalar>,
    origin: Point3<RayScalar>,
    normal: Vector3<RayScalar>,
    t: RayScalar,
}
type MaterialIndex = u16;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CubeMaterialIndex {
    Solid {
        index: MaterialIndex,
    },
    Translucent {
        index: MaterialIndex,
        density: MaterialIndex,
    },
}
impl CubeMaterialIndex {
    pub fn new_solid(index: MaterialIndex) -> Self {
        Self::Solid { index }
    }
    pub fn new_translucent(index: MaterialIndex, density: RayScalar) -> Self {
        Self::Translucent {
            index,
            density: (density * MaterialIndex::MAX as RayScalar) as MaterialIndex,
        }
    }
    pub fn new_air() -> Self {
        Self::Solid {
            index: MaterialIndex::MAX,
        }
    }
    pub fn is_solid(&self) -> bool {
        match self {
            Self::Translucent { index, .. } => *index != MaterialIndex::MAX,
            Self::Solid { index } => *index != MaterialIndex::MAX,
        }
    }
    pub fn is_air(&self) -> bool {
        !self.is_solid()
    }
}

impl Solid for CubeMaterialIndex {
    fn solid(&self) -> CubeType {
        match self {
            Self::Solid { index } => {
                if *index == MaterialIndex::MAX {
                    CubeType::Air
                } else {
                    CubeType::Solid
                }
            }
            Self::Translucent { index, density } => {
                if *index == MaterialIndex::MAX {
                    CubeType::Air
                } else {
                    CubeType::Translucent {
                        density: *density as RayScalar / MaterialIndex::MAX as RayScalar,
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct CubeMaterial {
    material: Lambertian,
    color: RgbColor,
}
impl CubeMaterial {
    pub fn distance(&self, other: &Self) -> RayScalar {
        self.color.distance(&other.color) as RayScalar
    }
    pub fn color(&self) -> RgbColor {
        self.color
    }
}
impl std::fmt::Debug for CubeMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cube Material")
            .field("color", &self.color)
            .finish()
    }
}
impl Material for CubeMaterial {
    fn name(&self) -> &'static str {
        "cube material"
    }

    fn scatter(&self, ray_in: Ray, record_in: &HitRay) -> Option<ScatterRecord> {
        self.material.scatter(ray_in, record_in)
    }

    fn scattering_pdf(
        &self,
        ray_in: Ray,
        record_in: &HitRecord,
        scattered_ray: Ray,
    ) -> Option<RayScalar> {
        self.material
            .scattering_pdf(ray_in, record_in, scattered_ray)
    }
}
impl CubeMaterial {
    pub fn new(color: RgbColor) -> Self {
        CubeMaterial {
            material: Lambertian {
                albedo: Box::new(SolidColor { color }),
            },
            color,
        }
    }
}
#[derive(Clone)]
pub struct VoxelWorld {
    solid_materials: Vec<CubeMaterial>,
    translucent_materials: Vec<CubeMaterial>,
    voxels: Voxels<CubeMaterialIndex>,
    x: i32,
    y: i32,
    z: i32,
}
impl VoxelWorld {
    /// gets witdh height and depth of voxel world
    pub(crate) fn size(&self) -> Vector3<u32> {
        self.voxels.size().map(|val| val as u32)
    }
    /// tries to get the material if it is in bounds of the world
    pub fn get(&self, position: Point3<u32>) -> Option<CubeMaterialIndex> {
        let size = self.size();
        if position.x < size.x && position.y < size.y && position.z < size.z {
            Some(self.voxels.get(
                position.x as usize,
                position.y as usize,
                position.z as usize,
            ))
        } else {
            None
        }
    }
    pub fn get_solid_material(&self, index: MaterialIndex) -> Option<CubeMaterial> {
        if (index as usize) < self.solid_materials.len() {
            Some(self.solid_materials[index as usize].clone())
        } else {
            None
        }
    }
    pub fn new(
        solid_materials: Vec<CubeMaterial>,
        translucent_materials: Vec<CubeMaterial>,
        x: i32,
        y: i32,
        z: i32,
    ) -> Self {
        Self {
            solid_materials,
            translucent_materials,
            voxels: Voxels::new(
                x as usize,
                y as usize,
                z as usize,
                CubeMaterialIndex::new_air(),
            ),
            x,
            y,
            z,
        }
    }
    pub fn update(&mut self, x: isize, y: isize, z: isize, val: CubeMaterialIndex) {
        match val {
            CubeMaterialIndex::Solid { index } => {
                if index == MaterialIndex::MAX || (index as usize) < self.solid_materials.len() {
                    self.voxels.update(x, y, z, val)
                } else {
                    error!("invalid cube material index: {}", index)
                }
            }
            CubeMaterialIndex::Translucent { index, .. } => {
                if index == MaterialIndex::MAX
                    || (index as usize) < self.translucent_materials.len()
                {
                    self.voxels.update(x, y, z, val);
                } else {
                    error!("invalid cube material index: {}", index)
                }
            }
        };
    }
    pub fn in_world(&self, x: isize, y: isize, z: isize) -> bool {
        x >= 0
            && x < self.x as isize
            && y >= 0
            && y < self.y as isize
            && z >= 0
            && z < self.z as isize
    }
    fn check_x(
        &self,
        ray: &Ray,
        t_min: RayScalar,
        t_max: RayScalar,
        x: RayScalar,
        normal: Vector3<RayScalar>,
    ) -> Option<CheckRes> {
        let t = (x - ray.origin.x) / ray.direction.x;
        if t >= t_min && t <= t_max {
            let pos = ray.origin + ray.direction * t;

            if pos.y >= 0.0
                && pos.y <= self.y as RayScalar
                && pos.z >= 0.0
                && pos.z <= self.z as RayScalar
            {
                Some(CheckRes {
                    direction: ray.direction,
                    origin: pos,
                    normal,
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
        t_min: RayScalar,
        t_max: RayScalar,
        y: RayScalar,
        normal: Vector3<RayScalar>,
    ) -> Option<CheckRes> {
        let t = (y - ray.origin.y) / ray.direction.y;
        if t > t_min && t < t_max {
            let pos = ray.at(t);

            if pos.x > 0.0
                && pos.x < self.x as RayScalar
                && pos.z > 0.0
                && pos.z < self.z as RayScalar
            {
                Some(CheckRes {
                    direction: ray.direction,
                    origin: pos,
                    normal,
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
        t_min: RayScalar,
        t_max: RayScalar,
        z: RayScalar,
        normal: Vector3<RayScalar>,
    ) -> Option<CheckRes> {
        let t = (z - ray.origin.z) / ray.direction.z;
        if t > t_min && t < t_max {
            let pos = ray.at(t);

            if pos.x >= 0.0
                && pos.x <= self.x as RayScalar
                && pos.y >= 0.0
                && pos.y <= self.y as RayScalar
            {
                Some(CheckRes {
                    direction: ray.direction,
                    origin: pos,
                    normal,
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
        hit: HitResult<CubeMaterialIndex>,
        t_min: RayScalar,
        t_max: RayScalar,
    ) -> Option<HitRecord> {
        match hit {
            HitResult::Hit {
                position,
                normal,
                voxel,
            } => {
                let dist = ray.origin - position;
                let t =
                    Vector3::new(dist.x, dist.y, dist.z).magnitude() / ray.direction.magnitude();
                if (t > t_min && t < t_max) && t >= 0.0 {
                    Some(HitRecord::new_ref(
                        ray,
                        position,
                        normal,
                        t,
                        Point2::new(0.0, 0.0),
                        match voxel {
                            CubeMaterialIndex::Solid { index } => {
                                &self.solid_materials[index as usize]
                            }
                            CubeMaterialIndex::Translucent { index, .. } => {
                                &self.translucent_materials[index as usize]
                            }
                        },
                    ))
                } else {
                    None
                }
            }
            HitResult::DidNotHit => None,
        }
    }
}

impl Hittable for VoxelWorld {
    fn hit(&self, ray: &Ray, t_min: RayScalar, t_max: RayScalar) -> Option<HitRecord> {
        let aabb = self.bounding_box(t_min, t_max).expect("failed to get aabb");
        if aabb.contains_point(ray.origin) {
            let hit_res = self.voxels.trace_voxels(
                Point3::new(ray.origin.x, ray.origin.y, ray.origin.z),
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
                self.x as RayScalar,
                Vector3::new(1.0, 0.0, 0.0),
            ),
            self.check_y(ray, t_min, t_max, 0.0, Vector3::new(0.0, -1.0, 0.0)),
            self.check_y(
                ray,
                t_min,
                t_max,
                self.y as RayScalar,
                Vector3::new(0.0, 1.0, 0.0),
            ),
            self.check_z(ray, t_min, t_max, 0.0, Vector3::new(0.0, 0.0, 1.0)),
            self.check_z(
                ray,
                t_min,
                t_max,
                self.z as RayScalar,
                Vector3::new(0.0, 0.0, 1.0),
            ),
        ];
        let mut min_dist = RayScalar::MAX;

        let mut min_index = usize::MAX;
        for i in 0..solutions.len() {
            if let Some(check) = solutions[i].as_ref() {
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
            let mut idx = s.origin.map(|v| v.floor() as usize);
            if idx.x == self.x as usize {
                idx.x = self.x as usize - 1;
            }
            if idx.y == self.y as usize {
                idx.y = self.y as usize - 1;
            }
            if idx.z == self.z as usize {
                idx.z = self.z as usize - 1;
            }

            let voxel = self.voxels.get(idx.x, idx.y, idx.z);
            if !voxel.is_solid() {
                self.manage_hit_res(
                    ray,
                    self.voxels.trace_voxels(s.origin, s.direction),
                    t_min,
                    t_max,
                )
            } else {
                match voxel {
                    CubeMaterialIndex::Solid { index } => Some(HitRecord::new_ref(
                        ray,
                        s.origin,
                        s.normal,
                        s.t,
                        Point2::new(0.0, 0.0),
                        &self.solid_materials[index as usize],
                    )),
                    CubeMaterialIndex::Translucent { index, density } => {
                        let n = step_translucent(
                            s.origin,
                            s.direction.normalize(),
                            density as RayScalar / MaterialIndex::MAX as RayScalar,
                        );
                        if let Some(next) = n {
                            Some(HitRecord::new_ref(
                                ray,
                                next,
                                s.normal,
                                s.t,
                                Point2::new(0.0, 0.0),
                                &self.translucent_materials[index as usize],
                            ))
                        } else {
                            let hit_res = self.voxels.trace_voxels(s.origin, s.direction);
                            self.manage_hit_res(ray, hit_res, t_min, t_max)
                        }
                    }
                }
            }
        } else {
            None
        }
    }

    fn bounding_box(&self, _time_0: RayScalar, _time_1: RayScalar) -> Option<Aabb> {
        Some(Aabb {
            minimum: Point3::new(0.0, 0.0, 0.0),
            maximum: Point3::new(
                self.x as RayScalar,
                self.y as RayScalar,
                self.z as RayScalar,
            ),
        })
    }

    fn prob(&self, _ray: Ray) -> RayScalar {
        todo!()
    }

    fn generate_ray_in_area(&self, _origin: Point3<RayScalar>, _time: RayScalar) -> RayAreaInfo {
        todo!()
    }
    fn name(&self) -> String {
        "Voxel World".to_string()
    }
}

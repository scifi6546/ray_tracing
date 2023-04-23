pub(crate) use super::parallel_image::{
    image_channel, ImageReceiver, ImageSender, ParallelImagePart, RayTracerMessage,
};
pub use super::parallel_image::{ParallelImage, ParallelImageCollector};
pub use base_lib::{clamp, RgbColor};
pub use cgmath;
use cgmath::{num_traits::FloatConst, prelude::*};
//use cgmath::num_traits::*;
pub(crate) use cgmath::{Point2, Point3, Vector3};
pub use log::{error, info, warn};

use std::{cmp::PartialOrd, fmt::*, ops::Div, path::Path};
pub fn rand_f32(min: f32, max: f32) -> f32 {
    rand::random::<f32>() * (max - min) + min
}
pub fn rand_u32(min: u32, max: u32) -> u32 {
    (rand::random::<u32>() % (max - min)) + min
}
pub fn p_min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}
pub fn p_max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        a
    } else {
        b
    }
}

pub struct OrthoNormalBasis {
    pub axis: [Vector3<f32>; 3],
}
impl OrthoNormalBasis {
    pub fn build_from_w(n: Vector3<f32>) -> Self {
        let w = n.normalize();
        let a = if w.x.abs() > 0.9 {
            Vector3::new(0.0, 1.0, 0.0)
        } else {
            Vector3::new(1.0, 0.0, 0.0)
        };
        let v = w.cross(a).normalize();
        let u = w.cross(v);
        Self { axis: [u, v, w] }
    }
    pub fn local(&self, a: Vector3<f32>) -> Vector3<f32> {
        a.x * self.u() + a.y * self.v() + a.z * self.w()
    }
    pub fn u(&self) -> Vector3<f32> {
        self.axis[0]
    }
    pub fn v(&self) -> Vector3<f32> {
        self.axis[1]
    }
    pub fn w(&self) -> Vector3<f32> {
        self.axis[2]
    }
}
impl std::ops::Index<usize> for OrthoNormalBasis {
    type Output = Vector3<f32>;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index <= 2);
        &self.axis[index]
    }
}
impl std::ops::IndexMut<usize> for OrthoNormalBasis {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index <= 2);
        &mut self.axis[index]
    }
}
pub fn random_cosine_direction() -> Vector3<f32> {
    let r1 = rand_f32(0.0, 1.0);
    let r2 = rand_f32(0.0, 1.0);
    let z = (1.0 - r2).sqrt();
    let phi = 2.0 * f32::PI() * r1;
    let x = phi.cos() * r2.sqrt();
    let y = phi.sin() * r2.sqrt();
    Vector3 { x, y, z }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
    pub time: f32,
}
impl Ray {
    pub fn at(&self, t: f32) -> Point3<f32> {
        self.origin + t * self.direction
    }
}
impl Display for Ray {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "(dir: <{},{},{}>, origin: <{},{},{}>, time: {})",
            self.direction.x,
            self.direction.y,
            self.direction.z,
            self.origin.x,
            self.origin.y,
            self.origin.z,
            self.time
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    pub fn test_rand() {
        for i in 0..10_000 {
            let r = rand_f32(0.0, 1.0);
            assert!(r >= 0.0);
            assert!(r <= 1.0);
        }
    }
    #[test]
    pub fn test_rand_u32() {
        for i in 100..10_000 {
            let r = rand_u32(0, i / 100);
            assert!(r <= i / 100 && r >= 0)
        }
    }
}

pub(crate) use super::parallel_image::{image_channel, ParallelImagePart, RayTracerMessage};
pub use super::parallel_image::{ParallelImage, ParallelImageCollector};
pub use cgmath;
use cgmath::{num_traits::FloatConst, prelude::*};
pub use rgb_color::RgbColor;
use std::iter::Iterator;
mod rgb_color;
pub(crate) use cgmath::{Point3, Vector3};
pub use log::{error, info, warn};
pub fn clamp<T: std::cmp::PartialOrd>(x: T, min: T, max: T) -> T {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}
pub struct IterBox {
    current_point: Point3<u32>,
    end_point: Point3<u32>,
}
impl Iterator for IterBox {
    type Item = Point3<u32>;
    fn next(&mut self) -> Option<Self::Item> {
        let return_point = self.current_point;
        if self.current_point.x >= self.end_point.x {
            return None;
        }

        if self.current_point.z + 1 < self.end_point.z {
            self.current_point.z += 1;
        } else if self.current_point.y + 1 < self.end_point.y {
            self.current_point.z = 0;
            self.current_point.y += 1;
        } else if self.current_point.x + 1 < self.end_point.x {
            self.current_point.y = 0;
            self.current_point.z = 0;
            self.current_point.x += 1;
        } else {
            self.current_point.x += 1;
        }
        Some(return_point)
    }
}

pub(crate) fn iter_box(end_point: Point3<u32>) -> IterBox {
    IterBox {
        end_point,
        current_point: Point3::new(0, 0, 0),
    }
}
use std::{cmp::PartialOrd, fmt::*};
/// Type that a ray uses.
pub type RayScalar = f64;
pub fn rand_scalar(min: RayScalar, max: RayScalar) -> RayScalar {
    rand::random::<RayScalar>() * (max - min) + min
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
    pub axis: [Vector3<RayScalar>; 3],
}
impl OrthoNormalBasis {
    pub fn build_from_w(n: Vector3<RayScalar>) -> Self {
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
    pub fn local(&self, a: Vector3<RayScalar>) -> Vector3<RayScalar> {
        a.x * self.u() + a.y * self.v() + a.z * self.w()
    }
    pub fn u(&self) -> Vector3<RayScalar> {
        self.axis[0]
    }
    pub fn v(&self) -> Vector3<RayScalar> {
        self.axis[1]
    }
    pub fn w(&self) -> Vector3<RayScalar> {
        self.axis[2]
    }
}
impl std::ops::Index<usize> for OrthoNormalBasis {
    type Output = Vector3<RayScalar>;

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
pub fn random_cosine_direction() -> Vector3<RayScalar> {
    let r1 = rand_scalar(0.0, 1.0);
    let r2 = rand_scalar(0.0, 1.0);
    let z = (1.0 - r2).sqrt();
    let phi = 2.0 * RayScalar::PI() * r1;
    let x = phi.cos() * r2.sqrt();
    let y = phi.sin() * r2.sqrt();
    Vector3 { x, y, z }
}

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Point3<RayScalar>,
    pub direction: Vector3<RayScalar>,
    pub time: RayScalar,
}
impl Ray {
    pub fn at(&self, t: RayScalar) -> Point3<RayScalar> {
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
        for _ in 0..10_000 {
            let r = rand_scalar(0.0, 1.0);
            assert!(r >= 0.0);
            assert!(r <= 1.0);
        }
    }
    #[test]
    pub fn test_rand_u32() {
        for i in 100..10_000 {
            let r = rand_u32(0, i / 100);
            assert!(r <= i / 100)
        }
    }
    #[test]
    fn iter_empty_box() {
        let num = iter_box(Point3::new(0, 0, 0)).count();
        assert_eq!(num, 0);
    }
    #[test]
    fn iter_4() {
        let num = iter_box(Point3::new(2, 2, 2)).collect::<Vec<_>>();

        let compare = vec![
            Point3::new(0, 0, 0),
            Point3::new(0, 0, 1),
            Point3::new(0, 1, 0),
            Point3::new(0, 1, 1),
            Point3::new(1, 0, 0),
            Point3::new(1, 0, 1),
            Point3::new(1, 1, 0),
            Point3::new(1, 1, 1),
        ];
        assert_eq!(compare, num);
    }
}

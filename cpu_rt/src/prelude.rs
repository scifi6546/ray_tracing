pub(crate) use super::parallel_image::{image_channel, ParallelImagePart, RayTracerMessage};
pub use super::parallel_image::{ParallelImage, ParallelImageCollector};
pub use cgmath;
use cgmath::{num_traits::FloatConst, prelude::*};
use std::ops::{Add, AddAssign, Div, Mul, Sub};

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

#[repr(C)]
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RgbColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}
impl RgbColor {
    pub const BLACK: Self = Self {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
    };
    pub const WHITE: Self = Self {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
    };
    pub fn distance(&self, other: &Self) -> f32 {
        ((self.red - other.red).powi(2)
            + (self.green - other.green).powi(2)
            + (self.blue - other.blue).powi(2))
        .sqrt()
    }
    pub fn magnitude_squared(&self) -> f32 {
        self.red.powi(2) + self.green.powi(2) + self.blue.powi(2)
    }
    pub fn new(red: f32, green: f32, blue: f32) -> Self {
        Self { red, green, blue }
    }
    pub fn random() -> Self {
        Self {
            red: rand::random(),
            green: rand::random(),
            blue: rand::random(),
        }
    }
    pub fn pow(&self, p: f32) -> Self {
        Self {
            red: self.red.powf(p),
            green: self.green.powf(p),
            blue: self.blue.powf(p),
        }
    }
    pub fn clamp(&self) -> RgbColor {
        Self {
            red: self.red.clamp(0.0, 1.0),
            green: self.green.clamp(0.0, 1.0),
            blue: self.blue.clamp(0.0, 1.0),
        }
    }
    pub fn exp(&self) -> Self {
        Self {
            red: self.red.exp(),
            green: self.green.exp(),
            blue: self.blue.exp(),
        }
    }
    pub fn as_rgb_u8(&self) -> [u8; 3] {
        let r = (clamp(self.red.sqrt(), 0.0, 1.0) * 255.0).round() as u8;
        let g = (clamp(self.green.sqrt(), 0.0, 1.0) * 255.0).round() as u8;
        let b = (clamp(self.blue.sqrt(), 0.0, 1.0) * 255.0).round() as u8;
        [r, g, b]
    }
    pub fn as_rgba_u8(&self) -> [u8; 4] {
        let [r, g, b] = self.as_rgb_u8();
        [r, g, b, 0xff]
    }
    pub fn normalize(self) -> Self {
        let mag = (self.red.powi(2) + self.green.powi(2) + self.blue.powi(2)).sqrt();
        mag * self
    }
    pub fn is_nan(&self) -> bool {
        self.red.is_nan() || self.green.is_nan() || self.blue.is_nan()
    }
}
impl Mul<f32> for RgbColor {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            red: self.red * rhs,
            green: self.green * rhs,
            blue: self.blue * rhs,
        }
    }
}
impl Mul<f64> for RgbColor {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            red: self.red * rhs as f32,
            green: self.green * rhs as f32,
            blue: self.blue * rhs as f32,
        }
    }
}
impl Mul<RgbColor> for f32 {
    type Output = RgbColor;
    fn mul(self, rhs: RgbColor) -> Self::Output {
        rhs * self
    }
}
impl Div<f32> for RgbColor {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        Self {
            red: self.red / rhs,
            green: self.green / rhs,
            blue: self.blue / rhs,
        }
    }
}
impl Mul<RgbColor> for f64 {
    type Output = RgbColor;
    fn mul(self, rhs: RgbColor) -> Self::Output {
        rhs * self
    }
}
impl Div<f64> for RgbColor {
    type Output = Self;
    fn div(self, rhs: f64) -> Self::Output {
        Self {
            red: self.red / rhs as f32,
            green: self.green / rhs as f32,
            blue: self.blue / rhs as f32,
        }
    }
}
impl Mul for RgbColor {
    type Output = RgbColor;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            red: self.red * rhs.red,
            green: self.green * rhs.green,
            blue: self.blue * rhs.blue,
        }
    }
}
impl Add for RgbColor {
    type Output = RgbColor;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            red: self.red + rhs.red,
            green: self.green + rhs.green,
            blue: self.blue + rhs.blue,
        }
    }
}
impl Sub for RgbColor {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            red: self.red - rhs.red,
            green: self.green - rhs.green,
            blue: self.blue - rhs.blue,
        }
    }
}
impl AddAssign for RgbColor {
    fn add_assign(&mut self, rhs: Self) {
        self.red += rhs.red;
        self.green += rhs.green;
        self.blue += rhs.blue;
    }
}
impl std::iter::Sum for RgbColor {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(
            RgbColor {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
            },
            |acc, x| acc + x,
        )
    }
}

impl Display for RgbColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.red, self.green, self.blue)
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
}

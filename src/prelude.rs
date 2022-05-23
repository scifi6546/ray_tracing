use cgmath::{num_traits::*, prelude::*, Point2, Point3, Vector3};

use std::{
    cmp::PartialOrd,
    fmt::*,
    ops::{Add, AddAssign, Div, Mul},
};

pub fn clamp<T: std::cmp::PartialOrd>(x: T, min: T, max: T) -> T {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}
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

#[repr(C)]
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
    pub fn to_rgba_u8(&self) -> [u8; 4] {
        let r = (clamp(self.red.sqrt(), 0.0, 1.0) * 255.0).round() as u8;
        let g = (clamp(self.green.sqrt(), 0.0, 1.0) * 255.0).round() as u8;
        let b = (clamp(self.blue.sqrt(), 0.0, 1.0) * 255.0).round() as u8;
        [r, g, b, 0xff]
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
#[derive(Clone)]
pub struct RgbImage {
    pub buffer: Vec<RgbColor>,
    pub width: u32,
    pub height: u32,
}
impl RgbImage {
    pub fn new_black(width: u32, height: u32) -> Self {
        let buffer = (0..(width as usize * height as usize))
            .map(|_| RgbColor {
                red: 0.0,
                blue: 0.0,
                green: 0.0,
            })
            .collect();

        RgbImage {
            buffer,
            width,
            height,
        }
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn get_uv(&self, uv: Point2<f32>) -> RgbColor {
        let x = ((uv.x * (self.width() as f32 - 1.0)) as u32).clamp(0, self.width() - 1);
        let v = 1.0 - uv.y;
        let y = ((v * (self.height() as f32 - 1.0)) as u32).clamp(0, self.height() - 1);
        self.get_xy(x, y)
    }
    pub fn get_xy(&self, x: u32, y: u32) -> RgbColor {
        self.buffer[y as usize * self.width as usize + x as usize]
    }
    pub fn set_xy(&mut self, x: u32, y: u32, color: RgbColor) {
        self.buffer[y as usize * self.width as usize + x as usize] = color;
    }
    pub fn add_xy(&mut self, x: u32, y: u32, color: RgbColor) {
        self.buffer[y as usize * self.width as usize + x as usize] += color;
    }
}
impl Div<f32> for RgbImage {
    type Output = RgbImage;

    fn div(mut self, rhs: f32) -> Self::Output {
        Self {
            buffer: self.buffer.drain(..).map(|c| c / rhs).collect(),
            width: self.width,
            height: self.height,
        }
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
pub fn debug() -> bool {
    const DEBUG: bool = false;
    rand_u32(0, 1_000_000) == 0 && DEBUG
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

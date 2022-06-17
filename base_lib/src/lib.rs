use cgmath::{Point3, Vector3};
use std::{
    fmt::{Display, Formatter},
    ops::{Add, AddAssign, Div, Mul, Sub},
};
pub struct Scene {
    pub name: String,
    pub objects: Vec<Object>,
    pub camera: Camera,
    pub background: Background,
}
pub enum Background {
    Sky,
}
pub struct Camera {
    pub aspect_ratio: f32,
    pub fov: f32,
    pub look_at: Point3<f32>,
    pub origin: Point3<f32>,
    pub up_vector: Vector3<f32>,
    pub aperture: f32,
    pub focus_distance: f32,
    pub start_time: f32,
    pub end_time: f32,
}
pub enum Texture {
    ConstantColor(RgbColor),
}
pub enum Material {
    Light(Texture),
}

pub struct Object {
    pub shape: Shape,
    pub material: Material,
}
pub enum Shape {
    Sphere { radius: f32, origin: Point3<f32> },
}
pub fn clamp<T: std::cmp::PartialOrd>(x: T, min: T, max: T) -> T {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}
pub fn get_scenarios() -> Vec<(String, fn() -> Scene)> {
    vec![("from baselib".to_string(), || Scene {
        name: "test baselib scene".to_string(),
        objects: vec![Object {
            shape: Shape::Sphere {
                radius: 0.5,
                origin: Point3::new(0.0, 0.0, 0.0),
            },
            material: Material::Light(Texture::ConstantColor(RgbColor::new(
                200000000000.0,
                0.0,
                0.0,
            ))),
        }],
        background: Background::Sky,
        camera: Camera {
            aspect_ratio: 1.0,
            fov: 20.0,
            origin: Point3::new(10.0, 10.0, 10.0),
            look_at: Point3::new(0.0, 0.0, 0.0),
            up_vector: Vector3::new(0.0, 1.0, 0.0),
            aperture: 0.00001,
            focus_distance: 10.0,
            start_time: 0.0,
            end_time: 0.0,
        },
    })]
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
    pub const WHITE: Self = Self {
        red: 1.0,
        green: 1.0,
        blue: 1.0,
    };
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
    pub fn as_rgba_u8(&self) -> [u8; 4] {
        let r = (clamp(self.red.sqrt(), 0.0, 1.0) * 255.0).round() as u8;
        let g = (clamp(self.green.sqrt(), 0.0, 1.0) * 255.0).round() as u8;
        let b = (clamp(self.blue.sqrt(), 0.0, 1.0) * 255.0).round() as u8;
        [r, g, b, 0xff]
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
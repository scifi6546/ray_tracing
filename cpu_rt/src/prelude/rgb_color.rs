use super::clamp;
use std::ops::{Add, AddAssign, Div, Mul, Sub};
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

impl std::fmt::Display for RgbColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.red, self.green, self.blue)
    }
}

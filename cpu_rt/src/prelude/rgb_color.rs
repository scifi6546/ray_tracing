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
    pub const RED: Self = Self {
        red: 1.,
        green: 0.,
        blue: 0.,
    };
    pub const GREEN: Self = Self {
        red: 0.,
        green: 1.,
        blue: 0.,
    };
    pub const BLUE: Self = Self {
        red: 0.,
        green: 0.,
        blue: 1.,
    };
    pub fn distance(&self, other: Self) -> f32 {
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
    pub fn from_color_hex(s: &str) -> Self {
        let hex_str = s.strip_prefix("#").expect("invalid syntax");
        let mut total = 0u32;
        assert!(hex_str.len() >= 6);
        for (idx, char) in hex_str.to_lowercase().char_indices().take(6) {
            let number = match char {
                '0' => 0x0u32,
                '1' => 0x1u32,
                '2' => 0x2u32,
                '3' => 0x3u32,
                '4' => 0x4u32,
                '5' => 0x5u32,
                '6' => 0x6u32,
                '7' => 0x7u32,
                '8' => 0x8u32,
                '9' => 0x9u32,
                'a' => 0xAu32,
                'b' => 0xBu32,
                'c' => 0xCu32,
                'd' => 0xDu32,
                'e' => 0xEu32,
                'f' => 0xFu32,
                _ => panic!("invalid char: {}", char),
            };
            total += number * 16u32.pow(5 - idx as u32);
        }
        let red_u32 = (total & 0xff0000) >> 16;
        let green_u32 = (total & 0x00ff00) >> 8;
        let blue_u32 = total & 0x0000ff;
        Self {
            red: red_u32 as f32 / 255.,
            green: green_u32 as f32 / 255.,
            blue: blue_u32 as f32 / 255.,
        }
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
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn black() {
        let b = RgbColor::from_color_hex("#000000");
        assert!(b.distance(RgbColor::BLACK) < 0.001);
    }
    #[test]
    fn red() {
        let r = RgbColor::from_color_hex("#ff0000");
        println!("red: {}", r);
        assert!(r.distance(RgbColor::RED) < 0.001);
    }
    #[test]
    fn green() {
        let g = RgbColor::from_color_hex("#00ff00");
        assert!(g.distance(RgbColor::GREEN) < 0.001);
    }
    #[test]
    fn blue() {
        let b = RgbColor::from_color_hex("#0000ff");
        assert!(b.distance(RgbColor::BLUE) < 0.001);
    }
    #[test]
    fn white() {
        let w = RgbColor::from_color_hex("#ffffff");
        assert!(w.distance(RgbColor::WHITE) < 0.001);
    }
    #[test]
    fn all_digits() {
        let colors = [
            ("#000001", 1),
            ("#000002", 2),
            ("#000003", 3),
            ("#000004", 4),
            ("#000005", 5),
            ("#000006", 6),
            ("#000007", 7),
            ("#000008", 8),
            ("#000009", 9),
            ("#00000A", 10),
            ("#00000B", 11),
            ("#00000C", 12),
            ("#00000D", 13),
            ("#00000E", 14),
            ("#00000F", 15),
        ];
        for (color_str, blue) in colors {
            assert_eq!(
                RgbColor::from_color_hex(color_str),
                RgbColor {
                    red: 0.,
                    green: 0.,
                    blue: blue as f32 / 255.
                }
            )
        }
    }
}

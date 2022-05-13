use std::{
    cmp::PartialOrd,
    ops::{Add, AddAssign, Div, Mul},
};
pub fn clamp(x: f32, min: f32, max: f32) -> f32 {
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

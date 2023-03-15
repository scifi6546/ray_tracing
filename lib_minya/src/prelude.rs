pub use base_lib::{clamp, RgbColor};
pub use cgmath;
use cgmath::{num_traits::*, prelude::*, Point2, Point3, Vector3};
pub use log::info;
use std::{cmp::PartialOrd, fmt::*, ops::Div, path::Path};
use to_numpy::NumpyArray3D;

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

#[derive(Clone)]
pub struct RgbImage {
    pub buffer: Vec<RgbColor>,
    pub width: u32,
    pub height: u32,
}
impl NumpyArray3D for RgbImage {
    fn get(&self, item: [usize; 3]) -> f32 {
        let c = self.get_xy(item[0] as u32, item[1] as u32);
        match item[2] {
            0 => c.red,
            1 => c.green,
            2 => c.blue,
            _ => panic!("out of bounds"),
        }
    }

    fn shape(&self) -> [usize; 3] {
        [self.width() as usize, self.height() as usize, 3]
    }
}
impl RgbImage {
    pub fn to_image(&self, num_samples: usize) -> image::RgbImage {
        let normalized_buffer: Vec<RgbColor> = self
            .buffer
            .iter()
            .map(|c| *c / num_samples as f32)
            .collect();
        let mi = normalized_buffer
            .iter()
            .flat_map(|p| [p.red, p.green, p.blue])
            .fold(f32::MAX, |acc, x| acc.min(x));
        let ma = normalized_buffer
            .iter()
            .flat_map(|p| [p.red, p.green, p.blue])
            .fold(f32::MAX, |acc, x| acc.max(x));
        info!("image min: {}, image max: {}", mi, ma);
        let mut image = image::RgbImage::from_pixel(self.width, self.height, image::Rgb([0, 0, 0]));
        for x in 0..self.width() {
            for y in 0..self.height() {
                let color = self.get_xy(x, y) / num_samples as f32;

                let rgb = color.as_rgb_u8();
                let u8_color = image::Rgb(rgb);
                image.put_pixel(x, self.height() - y - 1, u8_color);
            }
        }
        return image;
    }
    pub fn save_image<P: AsRef<Path>>(&self, p: P, num_samples: usize) {
        let img = self.to_image(num_samples);
        img.save(p).expect("failed to save image");
    }
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
    pub fn get_clamped(&self, x: i32, y: i32) -> RgbColor {
        self.get_xy(
            (x.max(0).min(self.width as i32 - 1)) as u32,
            (y.max(0).min(self.height as i32 - 1)) as u32,
        )
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
    pub fn filter_nan(&mut self, replacement: RgbColor) {
        for x in 0..self.width() {
            for y in 0..self.height() {
                let val = self.get_xy(x, y);
                if val.is_nan() {
                    self.set_xy(x, y, replacement)
                }
            }
        }
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

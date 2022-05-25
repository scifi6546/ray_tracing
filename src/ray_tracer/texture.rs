use crate::prelude::*;
use crate::ray_tracer::rand_vec;
use cgmath::{InnerSpace, Point2, Point3, Vector3};

pub trait Texture {
    fn color(&self, uv: Point2<f32>, pos: Point3<f32>) -> RgbColor;
}
pub struct SolidColor {
    pub(crate) color: RgbColor,
}
impl Texture for SolidColor {
    fn color(&self, _uv: Point2<f32>, _pos: Point3<f32>) -> RgbColor {
        self.color
    }
}
pub struct CheckerTexture {
    pub odd: Box<dyn Texture>,
    pub even: Box<dyn Texture>,
}
impl Texture for CheckerTexture {
    fn color(&self, uv: Point2<f32>, pos: Point3<f32>) -> RgbColor {
        let sin = (10.0 * pos.x).sin() * (10.0 * pos.y).sin() * (10.0 * pos.z).sin();
        if sin < 0.0 {
            self.odd.color(uv, pos)
        } else {
            self.even.color(uv, pos)
        }
    }
}
pub struct Perlin {
    ran_float: [Vector3<f32>; Self::POINT_COUNT],
    perm_x: [usize; Self::POINT_COUNT],
    perm_y: [usize; Self::POINT_COUNT],
    perm_z: [usize; Self::POINT_COUNT],
}
impl Perlin {
    const POINT_COUNT: usize = 256;
    fn perlin_generate_perm() -> [usize; Self::POINT_COUNT] {
        let mut p = [0; Self::POINT_COUNT];
        #[allow(clippy::needless_range_loop)]
        for i in 0..Self::POINT_COUNT {
            p[i] = i;
        }
        Self::permute(&mut p, Self::POINT_COUNT);
        p
    }
    fn permute(a: &mut [usize; Self::POINT_COUNT], n: usize) {
        for i in (0..n).rev() {
            let target = rand_u32(0, i as u32 + 1) as usize;
            a.swap(i, target);
        }
    }
    fn trilinear_interp(c: &[[[Vector3<f32>; 2]; 2]; 2], u: f32, v: f32, w: f32) -> f32 {
        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);

        let mut accum = 0.0;
        #[allow(clippy::needless_range_loop)]
        for i in 0..2 {
            #[allow(clippy::needless_range_loop)]
            for j in 0..2 {
                #[allow(clippy::needless_range_loop)]
                for k in 0..2 {
                    let weight_v = Vector3::new(u - i as f32, v - j as f32, w - k as f32);
                    accum += (i as f32 * uu + (1 - i) as f32 * (1.0 - uu))
                        * (j as f32 * vv + (1 - j) as f32 * (1.0 - vv))
                        * (k as f32 * ww + (1 - k) as f32 * (1.0 - ww))
                        * c[i][j][k].dot(weight_v);
                }
            }
        }
        accum
    }

    fn noise(&self, point: Point3<f32>) -> f32 {
        let u = point.x - point.x.floor();
        let v = point.y - point.y.floor();
        let w = point.z - point.z.floor();

        let mut c = [[[Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }; 2]; 2]; 2];

        let i = point.x.floor() as i32;
        let j = point.y.floor() as i32;
        let k = point.z.floor() as i32;
        #[allow(clippy::needless_range_loop)]
        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    c[di][dj][dk] = self.ran_float[self.perm_x[((i + di as i32) & 255) as usize]
                        ^ self.perm_y[((j + dj as i32) & 255) as usize]
                        ^ self.perm_z[((k + dk as i32) & 255) as usize]];
                }
            }
        }
        Self::trilinear_interp(&c, u, v, w)
    }
    pub fn turbulence(&self, point: Point3<f32>, depth: u32) -> f32 {
        let mut acum = 0.0;
        let mut temp_point = point;
        let mut weight = 1.0;
        for _ in 0..depth {
            acum += weight * self.noise(temp_point);
            weight *= 0.5;
            temp_point *= 2.0;
        }
        acum.abs()
    }
    pub fn new() -> Self {
        let mut ran_float = [Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }; Self::POINT_COUNT];
        #[allow(clippy::needless_range_loop)]
        for i in 0..Self::POINT_COUNT {
            ran_float[i] = rand_vec().normalize();
        }
        Self {
            ran_float,
            perm_x: Self::perlin_generate_perm(),
            perm_y: Self::perlin_generate_perm(),
            perm_z: Self::perlin_generate_perm(),
        }
    }
}
impl Texture for Perlin {
    fn color(&self, _uv: Point2<f32>, pos: Point3<f32>) -> RgbColor {
        let f = self.turbulence(2.0 * pos, 7);

        RgbColor::new(f, f, f)
    }
}
pub struct ImageTexture {
    texture: RgbImage,
}
impl ImageTexture {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Self {
        let reader = image::open(path).expect("failed to read image").into_rgb8();
        let mut texture = RgbImage::new_black(reader.width(), reader.height());
        for x in 0..reader.width() {
            for y in 0..reader.height() {
                let pixel = reader.get_pixel(x, y);
                texture.set_xy(
                    x,
                    y,
                    RgbColor::new(
                        pixel.0[0] as f32 / 255.0,
                        pixel.0[1] as f32 / 255.0,
                        pixel.0[2] as f32 / 255.0,
                    ),
                );
            }
        }
        Self { texture }
    }
}
impl Texture for ImageTexture {
    fn color(&self, uv: Point2<f32>, _pos: Point3<f32>) -> RgbColor {
        self.texture.get_uv(uv)
    }
}
pub struct DebugV {}
impl Texture for DebugV {
    fn color(&self, uv: Point2<f32>, _pos: Point3<f32>) -> RgbColor {
        RgbColor::new(uv.y, uv.y, uv.y)
    }
}

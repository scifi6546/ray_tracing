use crate::prelude::*;
use cgmath::{Point2, Point3};

pub trait Texture {
    fn color(&self, uv: Point2<f32>, pos: Point3<f32>) -> RgbColor;
}
pub struct SolidColor {
    pub(crate) color: RgbColor,
}
impl Texture for SolidColor {
    fn color(&self, uv: Point2<f32>, pos: Point3<f32>) -> RgbColor {
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
    ran_float: [f32; Self::POINT_COUNT],
    perm_x: [usize; Self::POINT_COUNT],
    perm_y: [usize; Self::POINT_COUNT],
    perm_z: [usize; Self::POINT_COUNT],
}
impl Perlin {
    const POINT_COUNT: usize = 256;
    fn perlin_generate_perm() -> [usize; Self::POINT_COUNT] {
        let mut p = [0; Self::POINT_COUNT];
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
    fn trilinear_interp(c: &[[[f32; 2]; 2]; 2], u: f32, v: f32, w: f32) -> f32 {
        let u = u * u * (3.0 - 2.0 * u);
        let v = v * v * (3.0 - 2.0 * v);
        let w = w * w * (3.0 - 2.0 * w);
        let mut accum = 0.0;
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    accum += (i as f32 * u + (1 - i) as f32 * (1.0 - u))
                        * (j as f32 * v + (1 - j) as f32 * (1.0 - v))
                        * (k as f32 * w + (1 - k) as f32 * (1.0 - w))
                        * c[i][j][k];
                }
            }
        }
        accum
    }

    fn noise(&self, point: Point3<f32>) -> f32 {
        let u = point.x - point.x.floor();
        let v = point.y - point.y.floor();
        let w = point.z - point.z.floor();

        let mut c = [[[0.0f32; 2]; 2]; 2];

        let i = point.x.floor() as i32;
        let j = point.y.floor() as i32;
        let k = point.z.floor() as i32;

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
    pub fn new() -> Self {
        let mut ran_float = [0.0; Self::POINT_COUNT];
        for i in 0..Self::POINT_COUNT {
            ran_float[i] = rand_f32(0.0, 1.0);
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
    fn color(&self, uv: Point2<f32>, pos: Point3<f32>) -> RgbColor {
        let f = self.noise(20.0 * pos);
        RgbColor::new(f, f, f)
    }
}

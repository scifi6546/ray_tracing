pub mod prelude;
pub mod ray_tracer;
use cgmath::{prelude::*, Vector3};

use prelude::*;
pub fn reflect(v: Vector3<f32>, normal: Vector3<f32>) -> Vector3<f32> {
    v - 2.0 * v.dot(normal) * normal
}
pub struct Image {
    buffer: Vec<u8>,
    width: u32,
    height: u32,
}
impl Image {
    pub fn from_rgb_image(image: &RgbImage) -> Self {
        let buffer = image.buffer.iter().flat_map(|c| c.as_rgba_u8()).collect();
        Self {
            buffer,
            width: image.width,
            height: image.height,
        }
    }
    pub fn new(buffer: Vec<u8>, width: u32, height: u32) -> Self {
        assert_eq!(buffer.len(), width as usize * height as usize * 4);
        Self {
            buffer,
            width,
            height,
        }
    }
    pub fn from_fn(ctor: fn(u32, u32) -> [u8; 4], width: u32, height: u32) -> Self {
        Self {
            buffer: (0..width)
                .flat_map(|y| (0..height).flat_map(move |x| ctor(x, y)))
                .collect(),
            width,
            height,
        }
    }
    pub fn set_xy(&mut self, x: u32, y: u32, pixel: [u8; 4]) {
        let offset = (self.width * y + x) * 4;
        for i in 0..4 {
            self.buffer[(offset + i) as usize] = pixel[i as usize];
        }
    }
    pub fn set_xy_color(&mut self, x: u32, y: u32, pixel: RgbColor) {
        self.set_xy(x, y, pixel.as_rgba_u8());
    }
    pub fn make_texture(&self, ctx: &mut miniquad::Context) -> miniquad::Texture {
        miniquad::Texture::from_rgba8(ctx, self.width as u16, self.height as u16, &self.buffer)
    }
}

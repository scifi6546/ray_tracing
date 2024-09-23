mod parallel_image;
pub mod prelude;
pub mod ray_tracer;

use cgmath::{prelude::*, Vector3};

use prelude::*;
pub fn reflect(v: Vector3<RayScalar>, normal: Vector3<RayScalar>) -> Vector3<RayScalar> {
    v - 2.0 * v.dot(normal) * normal
}
pub struct Image {
    buffer: Vec<u8>,
    width: u32,
    height: u32,
}
impl Image {
    pub fn from_parallel_image(image: &ParallelImage) -> Self {
        let mut buffer = vec![0u8; 4 * image.width() * image.height()];
        for x in 0..image.width() {
            for y in 0..image.height() {
                let c = image.get_xy(x, y).as_rgba_u8();
                buffer[x * 4 + y * image.width() * 4] = c[0];
                buffer[x * 4 + y * image.width() * 4 + 1] = c[1];
                buffer[x * 4 + y * image.width() * 4 + 2] = c[2];
                buffer[x * 4 + y * image.width() * 4 + 3] = c[3];
            }
        }

        Self {
            buffer,
            width: image.width() as u32,
            height: image.height() as u32,
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
    /// gets the width of the image
    pub fn width(&self) -> u32 {
        self.width
    }
    /// gets the height of the image
    pub fn height(&self) -> u32 {
        self.height
    }
    /// gets the buffer in the form rgba u8
    pub fn buffer_rgba8(&self) -> &[u8] {
        &self.buffer
    }
}

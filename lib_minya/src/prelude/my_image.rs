use crate::prelude::*;
use std::ops::Add;
use to_numpy::NumpyArray3D;
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
    pub fn down_sample(&self) -> Self {
        let new_width = self.width() / 2;
        let new_height = self.height() / 2;
        let mut img = Self::new_black(new_width, new_height);

        for y in 0..new_height {
            for x in 0..new_width {
                let x0y0 = self.get_xy(x * 2, y * 2);
                let x1y0 = self.get_xy((x * 2 + 1).min(self.width() - 1), y * 2);
                let x1y1 = self.get_xy(
                    (x * 2 + 1).min(self.width() - 1),
                    (y * 2 + 1).min(self.height() - 1),
                );
                let x0y1 = self.get_xy(x * 2, (y * 2 + 1).min(self.height() - 1));
                let avg = (x0y0 + x1y0 + x1y1 + x0y1) / 4.0;

                img.set_xy(x, y, avg);
            }
        }
        img
    }
    /// Gets nearest neighbor x: [0,1] y: [0,1]
    pub fn get_nearest(&self, x: f32, y: f32) -> RgbColor {
        assert!(self.width() > 1);
        assert!(self.height() > 1);
        let x_pixel = (x * (self.width() - 1) as f32)
            .min(self.width() as f32 - 1.0)
            .max(0.0);
        let y_pixel = (y * (self.height() - 1) as f32)
            .min(self.height() as f32 - 1.0)
            .max(0.0);
        let x0 = x_pixel.floor() as u32;
        let y0 = y_pixel.floor() as u32;

        let x1 = (x0 + 1).min(self.width() - 1);
        let y1 = (y0 + 1).min(self.height() - 1);
        let x0y0 = self.get_xy(x0, y0);
        let x1y0 = self.get_xy(x1, y0);
        let x1y1 = self.get_xy(x1, y1);
        let x0y1 = self.get_xy(x0, y1);

        let x_fract = x_pixel.fract();
        let y_fract = y_pixel.fract();

        let x0 = (1.0 - y_fract) * x0y0 + y_fract * x0y1;
        let x1 = (1.0 - y_fract) * x1y0 + y_fract * x1y1;
        (1.0 - x_fract) * x0 + x_fract * x1
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
impl Add<&RgbImage> for RgbImage {
    type Output = Self;

    fn add(mut self, rhs: &RgbImage) -> Self::Output {
        for y in 0..self.height() {
            let y_f = (y as f32 + 0.5) / (self.height() as f32 - 1.0);
            for x in 0..self.width() {
                let x_f = (x as f32 + 0.5) / (self.width() as f32 - 1.0);
                let pixel = rhs.get_nearest(x_f, y_f);
                self.add_xy(x, y, pixel)
            }
        }
        self
    }
}
impl Add<RgbImage> for RgbImage {
    type Output = Self;

    fn add(mut self, rhs: RgbImage) -> Self::Output {
        self.add(&rhs)
    }
}

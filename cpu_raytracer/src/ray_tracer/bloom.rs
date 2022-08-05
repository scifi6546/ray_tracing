use crate::prelude::*;
use log::debug;
use to_numpy::NumpyArray3D;
trait PostProcessingStage {
    fn process(&self, texture_in: &RgbImage) -> RgbImage;
}
struct GaussianBlur {
    amount: usize,
}
impl GaussianBlur {
    fn blur(texture: RgbImage) -> RgbImage {
        let weights = [0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216];

        let mut x_mod = texture.clone();
        x_mod.filter_nan(RgbColor::BLACK);
        for x in 0..texture.width() {
            for y in 0..texture.height() {
                let mut result = texture.get_xy(x, y) * weights[0];

                for i in 1..5 {
                    result += texture.get_clamped(x as i32 + i as i32, y as i32) * weights[i];
                    result += texture.get_clamped(x as i32 - i as i32, y as i32) * weights[i];
                }
                x_mod.set_xy(x, y, result);
            }
        }
        let mut y_mod = texture;
        for x in 0..y_mod.width() {
            for y in 0..y_mod.height() {
                let mut result = x_mod.get_xy(x, y) * weights[0];

                for i in 1..5 {
                    result += x_mod.get_clamped(x as i32, y as i32 + i as i32) * weights[i];
                    result += x_mod.get_clamped(x as i32, y as i32 - i as i32) * weights[i];
                }
                y_mod.set_xy(x, y, result);
            }
        }
        return y_mod;
    }
}
impl PostProcessingStage for GaussianBlur {
    fn process(&self, texture_in: &RgbImage) -> RgbImage {
        let mut mod_texture = texture_in.clone();
        for _ in 0..self.amount {
            mod_texture = Self::blur(mod_texture);
        }
        return mod_texture;
    }
}
struct SelectMinMag {
    min_mag: f32,
}
impl PostProcessingStage for SelectMinMag {
    fn process(&self, texture_in: &RgbImage) -> RgbImage {
        let mut out_texture = texture_in.clone();
        for x in 0..texture_in.width() {
            for y in 0..texture_in.height() {
                let rgb = texture_in.get_xy(x, y);

                if rgb.magnitude_squared() <= self.min_mag {
                    out_texture.set_xy(x, y, RgbColor::BLACK)
                }
            }
        }
        return out_texture;
    }
}
pub fn bloom(texture: &mut RgbImage) {
    let original_texture = texture.clone();

    let select = SelectMinMag { min_mag: 1.0 };
    let bright_texture = select.process(texture);

    let blur = GaussianBlur { amount: 10 };
    let mut bloom_texture = blur.process(&bright_texture);

    let gamma = 2.2;
    for x in 0..texture.width() {
        for y in 0..texture.height() {
            let hdr_color = original_texture.get_xy(x, y).clamp();
            let bloom_color = bloom_texture.get_xy(x, y).clamp() + hdr_color;
            let set_color = RgbColor::WHITE - (-1.0 * hdr_color * bloom_color).exp();
            let set_color = set_color.pow(1.0 / gamma);
            texture.set_xy(x, y, set_color);
        }
    }
}

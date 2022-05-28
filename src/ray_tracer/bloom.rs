use crate::prelude::*;
use log::debug;
trait PostProcessingStage {
    fn process(&self, texture_in: &RgbImage) -> RgbImage;
}
struct GaussianBlur {
    amount: usize,
}
impl GaussianBlur {
    fn blur(texture: &RgbImage) -> RgbImage {
        let weights = [0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216];
        let mut out_texture = texture.clone();
        for x in 0..texture.width() {
            for y in 0..texture.height() {
                let mut result = RgbColor::BLACK;
                for i in (-4i32)..5 {
                    for j in (-4i32)..5 {
                        let weight = weights[i.abs() as usize] * weights[j.abs() as usize];
                        let get_x = x as i32 + i;
                        let get_y = y as i32 + j;
                        if get_x >= 0
                            && get_x < texture.width() as i32
                            && get_y >= 0
                            && get_y < texture.height() as i32
                        {
                            result += texture.get_clamped(x as i32 + i, y as i32 + j) * weight;
                        } else {
                            result += texture.get_clamped(x as i32 + i, y as i32 + j) * weight;
                        }
                    }
                }

                out_texture.set_xy(x, y, result);
            }
        }
        return out_texture;
    }
}
impl PostProcessingStage for GaussianBlur {
    fn process(&self, texture_in: &RgbImage) -> RgbImage {
        let mut mod_texture = texture_in.clone();
        for _ in 0..self.amount {
            let t_texture = Self::blur(&mod_texture);
            mod_texture = t_texture;
        }
        return mod_texture;
    }
}
pub fn bloom(texture: &mut RgbImage) {
    let original_texture = texture.clone();
    let mut bright_texture = texture.clone();
    for x in 0..original_texture.width() {
        for y in 0..original_texture.height() {
            let rgb = original_texture.get_xy(x, y);

            if rgb.magnitude_squared() <= 1.0 {
                bright_texture.set_xy(x, y, RgbColor::BLACK)
            }
        }
    }
    let texture_copy = texture.clone();
    let blur = GaussianBlur { amount: 10 };
    let mut bloom_texture = blur.process(&bright_texture);

    let bloom_texture = bloom_texture.clone();
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
    return;
}

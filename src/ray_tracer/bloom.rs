use crate::prelude::*;
use log::debug;
pub fn bloom(texture: &mut RgbImage) {
    let original_texture = texture.clone();
    let weights = [0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216];
    for x in 0..original_texture.width() {
        for y in 0..original_texture.height() {
            let rgb = original_texture.get_xy(x, y);

            if rgb.magnitude_squared() <= 1.0 {
                texture.set_xy(x, y, RgbColor::BLACK)
            }
        }
    }
    let texture_copy = texture.clone();
    for x in 0..texture_copy.width() {
        for y in 0..texture_copy.height() {
            let mut result = RgbColor::BLACK;
            for i in (-4i32)..5 {
                for j in (-4i32)..5 {
                    let weight = weights[i.abs() as usize] * weights[j.abs() as usize];
                    let get_x = x as i32 + i;
                    let get_y = y as i32 + j;
                    if get_x >= 0
                        && get_x < texture_copy.width() as i32
                        && get_y >= 0
                        && get_y < texture_copy.height() as i32
                    {
                        result += texture_copy.get_clamped(x as i32 + i, y as i32 + j) * weight;
                    }
                }
            }

            texture.set_xy(x, y, result);
        }
    }

    let bloom_texture = texture.clone();
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

use super::Ray;
use crate::prelude::*;
use cgmath::prelude::*;
pub trait Background: Send {
    fn color(&self, ray: Ray) -> RgbColor;
}
pub struct Sky {
    pub intensity: f32,
}
impl Background for Sky {
    fn color(&self, ray: Ray) -> RgbColor {
        let unit = ray.direction.normalize();
        let t = 0.5 * (unit.y + 1.0);
        let color = (1.0 - t)
            * RgbColor {
                red: 1.0,
                blue: 1.0,
                green: 1.0,
            }
            + t * RgbColor {
                red: 0.5,
                green: 0.7,
                blue: 1.0,
            };
        self.intensity * color
    }
}
impl Default for Sky {
    fn default() -> Self {
        Self { intensity: 1.0 }
    }
}
pub struct ConstantColor {
    pub color: RgbColor,
}
impl Background for ConstantColor {
    fn color(&self, _ray: Ray) -> RgbColor {
        self.color
    }
}

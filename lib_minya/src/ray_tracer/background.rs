use super::{sun::Sun, Ray};
use crate::prelude::*;
use cgmath::prelude::*;
use cgmath::Vector3;
use dyn_clone::DynClone;

pub trait Background: Send + Sync + DynClone {
    fn color(&self, ray: Ray) -> RgbColor;
}
#[derive(Clone)]
pub struct Sky {
    pub intensity: RayScalar,
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
#[derive(Clone)]
pub struct SunSky {
    pub intensity: RayScalar,
    sun_radius: RayScalar,
    sun_theta: RayScalar,
    sun_phi: RayScalar,
    sun_brightness: RayScalar,
}
impl SunSky {
    pub fn new(sun: Sun, intensity: RayScalar, sun_brightness: RayScalar) -> Self {
        Self {
            intensity,
            sun_radius: sun.radius,
            sun_theta: sun.theta,
            sun_phi: sun.phi,
            sun_brightness,
        }
    }
}
impl Background for SunSky {
    fn color(&self, ray: Ray) -> RgbColor {
        let r = self.sun_phi.cos();

        let sun_ray = Vector3::new(
            r * self.sun_theta.cos(),
            self.sun_phi.sin(),
            r * self.sun_theta.sin(),
        );
        let sun_cos = sun_ray.dot(ray.direction.normalize());

        if sun_cos > self.sun_radius.cos() && sun_cos > 0.0 {
            self.sun_brightness * RgbColor::WHITE
        } else {
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
}
#[derive(Clone)]
pub struct ConstantColor {
    pub color: RgbColor,
}
impl Background for ConstantColor {
    fn color(&self, _ray: Ray) -> RgbColor {
        self.color
    }
}

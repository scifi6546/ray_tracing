use super::{new_demo, Metal, Object, SolidColor, Sphere, Transform, WorldInfo};
use crate::prelude::RayScalar;
use base_lib::RgbColor;
use cgmath::Point3;

pub fn metallic_smooth() -> WorldInfo {
    metalic_demo_fuzz(0.0)
}
pub fn metallic_rough() -> WorldInfo {
    metalic_demo_fuzz(0.8)
}
pub fn metalic_demo_fuzz(fuzz: RayScalar) -> WorldInfo {
    return new_demo(vec![Object::new(
        Box::new(Sphere {
            radius: 1.0,
            origin: Point3::new(0.0, 1.0, 0.0),
            material: Box::new(Metal {
                albedo: Box::new(SolidColor {
                    color: RgbColor::new(0.5, 0.1, 0.0),
                }),
                fuzz,
            }),
        }),
        Transform::identity(),
    )]);
}

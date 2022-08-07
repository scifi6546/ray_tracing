use super::{
    new_demo, Camera, CheckerTexture, ConstantColor, DebugV, Dielectric, DiffuseLight, Hittable,
    ImageTexture, Lambertian, Metal, Object, Perlin, RenderBox, Sky, SolidColor, Sphere, Transform,
    Translate, World, WorldInfo, XZRect, YZRect, IMAGE_HEIGHT, IMAGE_WIDTH,
};

use base_lib::RgbColor;
use cgmath::{prelude::*, Point3, Vector3};
use std::{cell::RefCell, rc::Rc};
pub fn metallic_smooth() -> WorldInfo {
    metalic_demo_fuzz(0.0)
}
pub fn metallic_rough() -> WorldInfo {
    metalic_demo_fuzz(0.8)
}
pub fn metalic_demo_fuzz(fuzz: f32) -> WorldInfo {
    return new_demo(Object::new(
        Rc::new(Sphere {
            radius: 1.0,
            origin: Point3::new(0.0, 1.0, 0.0),
            material: Rc::new(RefCell::new(Metal {
                albedo: Box::new(SolidColor {
                    color: RgbColor::new(0.5, 0.1, 0.0),
                }),
                fuzz,
            })),
        }),
        Transform::identity(),
    ));
}

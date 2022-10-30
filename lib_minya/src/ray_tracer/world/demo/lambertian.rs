use super::{new_demo, Lambertian, Object, SolidColor, Sphere, Transform, WorldInfo};

use base_lib::RgbColor;
use cgmath::Point3;
use std::{cell::RefCell, rc::Rc};
pub fn demo() -> WorldInfo {
    new_demo(vec![Object::new(
        Box::new(Sphere {
            radius: 1.0,
            origin: Point3::new(0.0, 1.0, 0.0),
            material: Box::new(Lambertian {
                albedo: Box::new(SolidColor {
                    color: RgbColor::new(0.5, 0.1, 0.0),
                }),
            }),
        }),
        Transform::identity(),
    )])
}

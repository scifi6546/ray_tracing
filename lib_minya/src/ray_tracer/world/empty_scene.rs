use super::{
    Camera, CheckerTexture, ConstantColor, DebugV, Dielectric, DiffuseLight, ImageTexture,
    Lambertian, Metal, Object, Perlin, RenderBox, SolidColor, Sphere, Transform, WorldInfo, XYRect,
    YZRect,
};
use crate::prelude::*;

use cgmath::{prelude::*, Point3, Vector3};
use std::{cell::RefCell, rc::Rc};

pub fn empty_scene() -> WorldInfo {
    let look_at = Point3::new(0.0f32, 0.0, -1.0);
    let origin = Point3::new(10.0f32, 3.0, 2.0);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    WorldInfo {
        objects: vec![],
        lights: vec![],
        background: Box::new(ConstantColor {
            color: RgbColor::new(0.00, 0.00, 0.00),
        }),
        camera: Camera::new(
            1.0,
            20.0,
            origin,
            look_at,
            Vector3::new(0.0, 1.0, 0.0),
            0.00001,
            focus_distance,
            0.0,
            0.0,
        ),
    }
}

pub mod cube_field;
pub mod lambertian;
pub mod metalic_demo;

use super::{
    Camera, DiffuseLight, Lambertian, Metal, Object, RenderBox, Sky, SolidColor, Sphere, Transform,
    WorldInfo, XZRect,
};
use crate::prelude::*;
use cgmath::{prelude::*, Point3, Vector3};

pub fn new_demo(mut special_item: Vec<Object>) -> WorldInfo {
    let look_at = Point3::new(0.0f32, 1.0, 0.0);
    let origin = Point3::new(10.0f32, 10.0, 2.0);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    let floor = Object::new(
        Box::new(XZRect::new(
            -5.0,
            5.0,
            -5.0,
            5.0,
            0.0,
            Box::new(Lambertian {
                albedo: Box::new(SolidColor {
                    color: RgbColor::new(0.5, 0.5, 0.5),
                }),
            }),
            false,
        )),
        Transform::identity(),
    );
    let light = Object::new(
        Box::new(Sphere {
            radius: 0.2,
            origin: Point3::new(0.0, 3.0, 1.0),
            material: Box::new(DiffuseLight {
                emit: Box::new(SolidColor {
                    color: 2000.0 * RgbColor::WHITE,
                }),
            }),
        }),
        Transform::identity(),
    );
    let mut objects = vec![floor, light.clone()];
    objects.append(&mut special_item);
    WorldInfo {
        objects,
        lights: vec![light],
        background: Box::new(Sky::default()),
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

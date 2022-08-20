pub mod cube_field;
pub mod lambertian;
pub mod metalic_demo;

use super::{
    Camera, DiffuseLight, Lambertian, Metal, Object, RenderBox, Sky, SolidColor, Sphere, Transform,
    WorldInfo, XZRect, IMAGE_HEIGHT, IMAGE_WIDTH,
};
use crate::prelude::*;
use cgmath::{prelude::*, Point3, Vector3};
use std::{cell::RefCell, rc::Rc};

pub fn new_demo(mut special_item: Vec<Object>) -> WorldInfo {
    let look_at = Point3::new(0.0f32, 1.0, 0.0);
    let origin = Point3::new(10.0f32, 10.0, 2.0);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };
    let floor = Object::new(
        Rc::new(XZRect {
            x0: -5.0,
            x1: 5.0,
            z0: -5.0,
            z1: 5.0,
            k: 0.0,
            material: Rc::new(RefCell::new(Lambertian {
                albedo: Box::new(SolidColor {
                    color: RgbColor::new(0.5, 0.5, 0.5),
                }),
            })),
        }),
        Transform::identity(),
    );

    let light = Object::new(
        Rc::new(Sphere {
            radius: 0.2,
            origin: Point3::new(0.0, 3.0, 1.0),
            material: Rc::new(RefCell::new(DiffuseLight {
                emit: Box::new(SolidColor {
                    color: 2000.0 * RgbColor::WHITE,
                }),
            })),
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
            IMAGE_WIDTH as f32 / IMAGE_HEIGHT as f32,
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

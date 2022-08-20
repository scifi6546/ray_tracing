use super::{
    Camera, Lambertian, Metal, Object, Sky, SolidColor, Sphere, Transform, WorldInfo, IMAGE_HEIGHT,
    IMAGE_WIDTH,
};
use crate::prelude::*;
use cgmath::{prelude::*, Point3, Vector3};
use std::{cell::RefCell, rc::Rc};
#[allow(dead_code)]
pub fn two_spheres() -> WorldInfo {
    let look_at = Point3::new(0.0f32, 0.0, -1.0);
    let origin = Point3::new(3.0f32, 3.0, 2.0);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    WorldInfo {
        objects: vec![
            Object::new(
                Rc::new(Sphere {
                    radius: 0.5,
                    origin: Point3 {
                        x: 0.0,
                        y: 0.0,
                        z: -1.0,
                    },
                    material: Rc::new(RefCell::new(Lambertian {
                        albedo: Box::new(SolidColor {
                            color: RgbColor {
                                red: 0.1,
                                green: 0.2,
                                blue: 0.5,
                            },
                        }),
                    })),
                }),
                Transform::identity(),
            ),
            Object::new(
                Rc::new(Sphere {
                    radius: 0.5,
                    origin: Point3 {
                        x: 1.0,
                        y: 0.0,
                        z: -1.0,
                    },
                    material: Rc::new(RefCell::new(Metal {
                        albedo: Box::new(SolidColor {
                            color: RgbColor::new(0.8, 0.6, 0.2),
                        }),
                        fuzz: 0.0,
                    })),
                }),
                Transform::identity(),
            ),
        ],
        lights: vec![],
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

use super::{
    Camera, CheckerTexture, ConstantColor, DebugV, Dielectric, DiffuseLight, ImageTexture,
    Lambertian, Metal, Object, Perlin, RenderBox, SolidColor, Sphere, Transform, WorldInfo, XYRect,
    YZRect, IMAGE_HEIGHT, IMAGE_WIDTH,
};
use crate::prelude::*;

use cgmath::{prelude::*, Point3, Vector3};
use std::{cell::RefCell, rc::Rc};

pub fn easy_scene() -> WorldInfo {
    let look_at = Point3::new(0.0f32, 0.0, -1.0);
    let origin = Point3::new(10.0f32, 3.0, 2.0);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };
    let light = Object::new(
        Box::new(XYRect {
            x0: -0.5,
            x1: 0.5,
            y0: -0.5 + 1.0,
            y1: 0.5 + 1.0,
            k: -2.3,
            material: Box::new(DiffuseLight {
                emit: Box::new(SolidColor {
                    color: 0.5 * RgbColor::new(0.0, 0.0, 1.0),
                }),
            }),
        }),
        Transform::identity(),
    );
    let yz_light = Object::new(
        Box::new(YZRect {
            y0: -0.5,
            y1: 0.5,
            z0: -0.5,
            z1: 0.5,
            k: -3.0,
            material: Box::new(DiffuseLight {
                emit: Box::new(SolidColor {
                    color: 0.5 * RgbColor::new(0.0, 1.0, 0.0),
                }),
            }),
        }),
        Transform::identity(),
    );
    let box_light = Object::new(
        Box::new(RenderBox::new(
            Point3::new(-0.2, -0.2 - 0.3, -0.2),
            Point3::new(0.2, 0.2 - 0.3, 0.2),
            Box::new(DiffuseLight {
                emit: Box::new(SolidColor {
                    color: 40000.0 * RgbColor::new(1.0, 0.0, 0.0),
                }),
            }),
        )),
        Transform::identity(),
    );
    let sphere_light = Object::new(
        Box::new(Sphere {
            radius: 0.5,
            origin: Point3 {
                x: 0.0,
                y: 1.5,
                z: -1.0,
            },
            material: Box::new(DiffuseLight {
                emit: Box::new(SolidColor {
                    color: RgbColor::new(4.0, 4.0, 4.0),
                }),
            }),
        }),
        Transform::identity(),
    );

    WorldInfo {
        objects: vec![
            Object::new(
                Box::new(Sphere {
                    radius: 100.0,
                    origin: Point3 {
                        x: 0.0,
                        y: -100.5,
                        z: -1.0,
                    },
                    material: Box::new(Lambertian {
                        albedo: Box::new(CheckerTexture {
                            even: Box::new(SolidColor {
                                color: RgbColor::new(0.5, 1.0, 0.0),
                            }),
                            odd: Box::new(Perlin::new()),
                        }),
                    }),
                }),
                Transform::identity(),
            ),
            Object::new(
                Box::new(Sphere {
                    radius: 0.5,
                    origin: Point3 {
                        x: 0.0,
                        y: 0.0,
                        z: -1.0,
                    },
                    material: Box::new(Lambertian {
                        albedo: Box::new(ImageTexture::new("./assets/earthmap.jpg")),
                    }),
                }),
                Transform::identity(),
            ),
            Object::new(
                Box::new(Sphere {
                    radius: 0.5,
                    origin: Point3 {
                        x: -1.0,
                        y: 0.0,
                        z: -1.0,
                    },
                    material: Box::new(Dielectric {
                        color: RgbColor::new(1.0, 0.8, 0.8),
                        index_refraction: 1.5,
                    }),
                }),
                Transform::identity(),
            ),
            Object::new(
                Box::new(Sphere {
                    radius: -0.45,
                    origin: Point3 {
                        x: -1.0,
                        y: 0.0,
                        z: 1.0,
                    },
                    material: Box::new(Dielectric {
                        color: RgbColor::new(1.0, 1.0, 1.0),
                        index_refraction: 1.5,
                    }),
                }),
                Transform::identity(),
            ),
            Object::new(
                Box::new(Sphere {
                    radius: 0.5,
                    origin: Point3 {
                        x: 1.0,
                        y: 0.0,
                        z: -1.0,
                    },
                    material: Box::new(Metal {
                        albedo: Box::new(DebugV {}),
                        fuzz: 0.0,
                    }),
                }),
                Transform::identity(),
            ),
            light.clone(),
            yz_light.clone(),
            box_light.clone(),
            sphere_light.clone(),
        ],
        lights: vec![light, yz_light, box_light, sphere_light],
        background: Box::new(ConstantColor {
            color: RgbColor::new(0.00, 0.00, 0.00),
        }),
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

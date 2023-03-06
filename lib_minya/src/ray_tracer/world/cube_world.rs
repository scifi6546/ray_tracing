use super::{
    Camera, ConstantColor, ConstantMedium, CubeWorld, DiffuseLight, FlipNormals, Isotropic,
    Lambertian, Object, RenderBox, SolidColor, Sphere, Transform, WorldInfo, XYRect, XZRect,
    YZRect,
};
use crate::prelude::*;
use cgmath::{prelude::*, Point3, Vector3};
use dyn_clone::clone_box;
use std::ops::Deref;
use std::{cell::RefCell, rc::Rc};

pub fn cube_world() -> WorldInfo {
    let look_at = Point3::new(0.0f32, 0.0, 0.0);
    let origin = Point3::new(-100.0f32, 100.0, 50.0);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };
    let green = Box::new(Lambertian {
        albedo: Box::new(SolidColor {
            color: RgbColor::new(0.12, 0.45, 0.15),
        }),
    });
    let red = Box::new(Lambertian {
        albedo: Box::new(SolidColor {
            color: RgbColor::new(0.65, 0.05, 0.05),
        }),
    });
    let light = Box::new(DiffuseLight {
        emit: Box::new(SolidColor {
            color: RgbColor::new(7.0, 7.0, 7.0),
        }),
    });

    let white = Box::new(Lambertian {
        albedo: Box::new(SolidColor {
            color: RgbColor::new(0.73, 0.73, 0.73),
        }),
    });
    let top_light = Object::new(
        Box::new(FlipNormals {
            item: Box::new(XZRect {
                x0: 113.0,
                x1: 443.0,
                z0: 127.0,
                z1: 423.0,
                k: 554.0,
                material: clone_box(light.deref()),
            }),
        }),
        Transform::identity(),
    );

    WorldInfo {
        objects: vec![Object::new(
            Box::new(CubeWorld::new(red, 10, 10, 10)),
            Transform::identity(),
        )],
        lights: vec![top_light],
        background: Box::new(ConstantColor {
            color: RgbColor::new(0.5, 0.5, 0.5),
        }),
        camera: Camera::new(
            1.0,
            40.0,
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

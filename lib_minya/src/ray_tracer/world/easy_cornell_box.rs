use super::{
    Camera, ConstantColor, DiffuseLight, FlipNormals, Lambertian, Object, SolidColor, Transform,
    WorldInfo, XYRect, XZRect, YZRect,
};
use crate::prelude::*;
use cgmath::{prelude::*, Point3, Vector3};
use std::{cell::RefCell, rc::Rc};

pub fn easy_cornell_box() -> WorldInfo {
    let look_at = Point3::new(278.0f32, 278.0, 0.0);
    let origin = Point3::new(278.0, 278.0, -800.0);
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
            color: RgbColor::new(15.0, 15.0, 15.0),
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
                x0: 213.0,
                x1: 343.0,
                z0: 227.0,
                z1: 332.0,
                k: 554.0,
                material: light.clone(),
            }),
        }),
        Transform::identity(),
    );
    /*
        let top_light = Object::new(
            Box::new(XZRect {
                x0: 213.0,
                x1: 343.0,
                z0: 227.0,
                z1: 332.0,
                k: 554.0,
                material: light,
            }),
            Transform::identity(),
        );
    */
    WorldInfo {
        objects: vec![
            Object::new(
                Box::new(YZRect {
                    y0: 0.0,
                    y1: 555.0,
                    z0: 0.0,
                    z1: 555.0,
                    k: 555.0,
                    material: green,
                }),
                Transform::identity(),
            ),
            Object::new(
                Box::new(YZRect {
                    y0: 0.0,
                    y1: 555.0,
                    z0: 0.0,
                    z1: 555.0,
                    k: 0.0,
                    material: red,
                }),
                Transform::identity(),
            ),
            Object::new(
                Box::new(XZRect {
                    x0: 0.0,
                    x1: 555.0,
                    z0: 0.0,
                    z1: 555.0,
                    k: 0.0,
                    material: white.clone(),
                }),
                Transform::identity(),
            ),
            Object::new(
                Box::new(XZRect {
                    x0: 0.0,
                    x1: 555.0,
                    z0: 0.0,
                    z1: 555.0,
                    k: 555.0,
                    material: white.clone(),
                }),
                Transform::identity(),
            ),
            Object::new(
                Box::new(XYRect {
                    x0: 0.0,
                    x1: 555.0,
                    y0: 0.0,
                    y1: 555.0,
                    k: 555.0,
                    material: white,
                }),
                Transform::identity(),
            ),
            top_light.clone(),
        ],
        lights: vec![top_light],
        background: Box::new(ConstantColor {
            color: RgbColor::new(0.0, 0.0, 0.0),
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

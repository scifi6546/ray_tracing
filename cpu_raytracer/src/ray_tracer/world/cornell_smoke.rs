use super::{
    Camera, ConstantColor, ConstantMedium, DiffuseLight, FlipNormals, Isotropic, Lambertian,
    Object, RenderBox, RotateY, SolidColor, Sphere, Transform, WorldInfo, XYRect, XZRect, YZRect,
    IMAGE_HEIGHT, IMAGE_WIDTH,
};
use crate::prelude::*;
use cgmath::{prelude::*, Point3, Vector3};
use std::{cell::RefCell, rc::Rc};

#[allow(dead_code)]
pub fn cornell_smoke() -> WorldInfo {
    let look_at = Point3::new(278.0f32, 278.0, 0.0);
    let origin = Point3::new(278.0, 278.0, -800.0);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };
    let green = Rc::new(RefCell::new(Lambertian {
        albedo: Box::new(SolidColor {
            color: RgbColor::new(0.12, 0.45, 0.15),
        }),
    }));
    let red = Rc::new(RefCell::new(Lambertian {
        albedo: Box::new(SolidColor {
            color: RgbColor::new(0.65, 0.05, 0.05),
        }),
    }));
    let light = Rc::new(RefCell::new(DiffuseLight {
        emit: Box::new(SolidColor {
            color: RgbColor::new(7.0, 7.0, 7.0),
        }),
    }));

    let white = Rc::new(RefCell::new(Lambertian {
        albedo: Box::new(SolidColor {
            color: RgbColor::new(0.73, 0.73, 0.73),
        }),
    }));
    let top_light = Object::new(
        Rc::new(FlipNormals {
            item: Rc::new(XZRect {
                x0: 113.0,
                x1: 443.0,
                z0: 127.0,
                z1: 423.0,
                k: 554.0,
                material: light,
            }),
        }),
        Transform::identity(),
    );
    WorldInfo {
        objects: vec![
            Object::new(
                Rc::new(YZRect {
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
                Rc::new(YZRect {
                    y0: 0.0,
                    y1: 555.0,
                    z0: 0.0,
                    z1: 555.0,
                    k: 0.0,
                    material: red,
                }),
                Transform::identity(),
            ),
            top_light.clone(),
            Object::new(
                Rc::new(XZRect {
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
                Rc::new(XZRect {
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
                Rc::new(XYRect {
                    x0: 0.0,
                    x1: 555.0,
                    y0: 0.0,
                    y1: 555.0,
                    k: 555.0,
                    material: white.clone(),
                }),
                Transform::identity(),
            ),
            Object::new(
                Rc::new(ConstantMedium::new(
                    Rc::new(RenderBox::new(
                        Point3::new(0.0, 0.0, 0.0),
                        Point3::new(165.0, 330.0, 165.0),
                        white.clone(),
                    )),
                    Rc::new(RefCell::new(Isotropic {
                        albedo: Box::new(SolidColor {
                            color: RgbColor::new(0.0, 0.0, 0.0),
                        }),
                    })),
                    0.01,
                )),
                Transform::translation(Vector3::new(265.0, 0.0, 295.0)),
            ),
            Object::new(
                Rc::new(ConstantMedium::new(
                    Rc::new(Sphere {
                        radius: 100.0,
                        origin: Point3::new(0.0, 0.0, 0.0),
                        material: white.clone(),
                    }),
                    Rc::new(RefCell::new(Isotropic {
                        albedo: Box::new(SolidColor {
                            color: RgbColor::new(0.5, 0.0, 0.0),
                        }),
                    })),
                    0.01,
                )),
                Transform::translation(Vector3::new(265.0, 500.0, 295.0)),
            ),
            Object::new(
                Rc::new(RotateY::new(
                    RenderBox::new(
                        Point3::new(0.0, 0.0, 0.0),
                        Point3::new(165.0, 165.0, 165.0),
                        white,
                    ),
                    -18.0,
                )),
                Transform::translation(Vector3::new(130.0, 0.0, 65.0)),
            ),
        ],
        lights: vec![top_light],
        background: Box::new(ConstantColor {
            color: RgbColor::new(0.0, 0.0, 0.0),
        }),
        camera: Camera::new(
            IMAGE_WIDTH as f32 / IMAGE_HEIGHT as f32,
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

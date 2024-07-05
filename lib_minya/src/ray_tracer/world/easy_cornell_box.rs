use super::{
    Camera, ConstantColor, DiffuseLight, Lambertian, Object, SolidColor, Transform, WorldInfo,
    XYRect, XZRect, YZRect,
};
use crate::prelude::*;
use cgmath::{prelude::*, Point3, Vector3};

pub fn easy_cornell_box() -> WorldInfo {
    let look_at = Point3::<RayScalar>::new(278.0, 278.0, 0.0);
    let origin = Point3::<RayScalar>::new(278.0, 278.0, -800.0);
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
        Box::new(XZRect::new(
            213.0,
            343.0,
            227.0,
            332.0,
            554.0,
            light.clone(),
            true,
        )),
        Transform::identity(),
    );

    WorldInfo {
        objects: vec![
            Object::new(
                Box::new(YZRect::new(0.0, 555.0, 0.0, 555.0, 555.0, green, true)),
                Transform::identity(),
            ),
            Object::new(
                Box::new(YZRect::new(0.0, 555.0, 0.0, 555.0, 0.0, red, false)),
                Transform::identity(),
            ),
            Object::new(
                Box::new(XZRect::new(
                    0.0,
                    555.0,
                    0.0,
                    555.0,
                    0.0,
                    white.clone(),
                    false,
                )),
                Transform::identity(),
            ),
            Object::new(
                Box::new(XZRect::new(
                    0.0,
                    555.0,
                    0.0,
                    555.0,
                    555.0,
                    white.clone(),
                    true,
                )),
                Transform::identity(),
            ),
            Object::new(
                Box::new(XYRect::new(0.0, 555.0, 0.0, 555.0, 555.0, white, true)),
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
        sun: None,
    }
}

use super::{
    Camera, Dielectric, DiffuseLight, Lambertian, Object, Sky, SolidColor, Sphere, Transform,
    WorldInfo, XZRect,
};

use base_lib::RgbColor;
use cgmath::{prelude::*, Point3, Vector3};

pub fn dielectric_no_refraction() -> WorldInfo {
    dielectric_demo(1.0)
}
pub fn dielectric_refraction() -> WorldInfo {
    dielectric_demo(1.5)
}
pub fn dielectric_demo(refraction: f32) -> WorldInfo {
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
    let l_sphere = Object::new(
        Box::new(Sphere {
            radius: 1.0,
            origin: Point3::new(0.0, 1.0, 0.0),
            material: Box::new(Dielectric {
                index_refraction: refraction,
                color: 0.8 * RgbColor::new(1.0, 1.0, 1.0),
            }),
        }),
        Transform::identity(),
    );
    let distorted_sphere = Object::new(
        Box::new(Sphere {
            radius: 1.0,
            origin: Point3::new(-1.0, 1.0, 1.0),
            material: Box::new(Lambertian {
                albedo: Box::new(SolidColor {
                    color: RgbColor::new(0.5, 0.1, 0.0),
                }),
            }),
        }),
        Transform::identity(),
    );
    let light = Object::new(
        Box::new(Sphere {
            radius: 0.2,
            origin: Point3::new(0.0, 3.0, 1.0),
            material: Box::new(DiffuseLight {
                emit: Box::new(SolidColor {
                    color: 2.0 * RgbColor::WHITE,
                }),
            }),
        }),
        Transform::identity(),
    );
    WorldInfo {
        objects: vec![floor, light.clone(), distorted_sphere, l_sphere],
        lights: vec![light],
        background: Box::new(Sky { intensity: 0.3 }),
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

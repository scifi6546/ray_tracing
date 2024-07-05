use super::{Camera, Lambertian, Object, Sky, SolidColor, Sphere, Transform, WorldInfo};
use crate::prelude::*;
use cgmath::{prelude::*, Point3, Vector3};

#[allow(dead_code)]
pub fn one_sphere() -> WorldInfo {
    let look_at = Point3::<RayScalar>::new(0.0, 0.0, -1.0);
    let origin = Point3::<RayScalar>::new(3.0, 3.0, 2.0);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    WorldInfo {
        objects: vec![Object::new(
            Box::new(Sphere {
                radius: 0.5,
                origin: Point3 {
                    x: 0.0,
                    y: 0.0,
                    z: -1.0,
                },
                material: Box::new(Lambertian {
                    albedo: Box::new(SolidColor {
                        color: RgbColor {
                            red: 0.1,
                            green: 0.2,
                            blue: 0.5,
                        },
                    }),
                }),
            }),
            Transform::identity(),
        )],
        lights: vec![],
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
        sun: None,
    }
}

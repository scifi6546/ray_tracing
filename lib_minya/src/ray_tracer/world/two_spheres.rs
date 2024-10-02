use super::{
    Camera, CameraInfo, Lambertian, Metal, Object, Sky, SolidColor, Sphere, Transform, WorldInfo,
};
use crate::prelude::*;
use cgmath::{prelude::*, Point3, Vector3};

#[allow(dead_code)]
pub fn two_spheres() -> WorldInfo {
    let look_at = Point3::<RayScalar>::new(0.0, 0.0, -1.0);
    let origin = Point3::<RayScalar>::new(3.0, 3.0, 2.0);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    WorldInfo {
        objects: vec![
            Object::new(
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
                        albedo: Box::new(SolidColor {
                            color: RgbColor::new(0.8, 0.6, 0.2),
                        }),
                        fuzz: 0.0,
                    }),
                }),
                Transform::identity(),
            ),
        ],
        lights: vec![],
        background: Box::new(Sky::default()),
        camera: Camera::new(CameraInfo {
            aspect_ratio: 1.0,
            fov: 20.,
            origin,
            look_at,
            up_vector: Vector3::unit_y(),
            aperture: 0.00001,
            focus_distance,
            start_time: 0.0,
            end_time: 0.0,
        }),
        sun: None,
    }
}

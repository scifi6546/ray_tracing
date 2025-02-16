use super::{
    Camera, CameraInfo, Dielectric, Lambertian, Metal, MovingSphere, Object, Sky, SolidColor,
    Sphere, Transform, WorldInfo,
};
use crate::prelude::*;
use cgmath::{prelude::*, Point3, Vector3};

pub fn random_scene() -> WorldInfo {
    let big: [Object; 4] = [
        Object::new(
            Box::new(Sphere {
                radius: 1000.0,
                origin: Point3::new(0.0, -1000.0, 1000.0),
                material: Box::new(Lambertian {
                    albedo: Box::new(SolidColor {
                        color: RgbColor {
                            red: 0.5,
                            green: 0.5,
                            blue: 0.5,
                        },
                    }),
                }),
            }),
            Transform::identity(),
        ),
        Object::new(
            Box::new(Sphere {
                radius: 1.0,
                origin: Point3::new(0.0, 1.0, 0.0),
                material: Box::new(Dielectric {
                    index_refraction: 1.5,
                    color: RgbColor::new(1.0, 1.0, 1.0),
                }),
            }),
            Transform::identity(),
        ),
        Object::new(
            Box::new(Sphere {
                radius: 1.0,
                origin: Point3::new(-4.0, 1.0, 0.0),
                material: Box::new(Lambertian {
                    albedo: Box::new(SolidColor {
                        color: RgbColor::new(0.4, 0.2, 0.1),
                    }),
                }),
            }),
            Transform::identity(),
        ),
        Object::new(
            Box::new(Sphere {
                radius: 1.0,
                origin: Point3::new(4.0, 1.0, 0.0),
                material: Box::new(Metal {
                    albedo: Box::new(SolidColor {
                        color: RgbColor::new(0.4, 0.2, 0.1),
                    }),
                    fuzz: 0.0,
                }),
            }),
            Transform::identity(),
        ),
    ];
    let objects = (-11..11)
        .flat_map(|a| {
            (-11..11).filter_map::<Object, _>(move |b| {
                let choose_mat = rand::random::<RayScalar>();
                let center = Point3::new(
                    a as RayScalar + 0.9 * rand::random::<RayScalar>(),
                    0.2,
                    b as RayScalar + 0.9 * rand::random::<RayScalar>(),
                );
                let check = center - Point3::new(4.0, 0.2, 0.0);
                if check.dot(check).sqrt() > 0.9 {
                    if choose_mat < 0.8 {
                        Some(Object::new(
                            Box::new(MovingSphere {
                                radius: 0.2,
                                center_0: center,
                                center_1: center + Vector3::new(0.0, rand_scalar(0.0, 0.5), 0.0),
                                time_0: 0.0,
                                time_1: 1.0,
                                material: Box::new(Lambertian {
                                    albedo: Box::new(SolidColor {
                                        color: RgbColor::random(),
                                    }),
                                }),
                            }),
                            Transform::identity(),
                        ))
                    } else if choose_mat < 0.95 {
                        Some(Object::new(
                            Box::new(Sphere {
                                radius: 0.2,
                                origin: center,
                                material: Box::new(Metal {
                                    albedo: Box::new(SolidColor {
                                        color: RgbColor::random(),
                                    }),
                                    fuzz: rand::random::<RayScalar>() * 0.5 + 0.5,
                                }),
                            }),
                            Transform::identity(),
                        ))
                    } else {
                        Some(Object::new(
                            Box::new(Sphere {
                                radius: 0.2,
                                origin: center,
                                material: Box::new(Dielectric {
                                    color: RgbColor::new(1.0, 1.0, 1.0),
                                    index_refraction: 1.5,
                                }),
                            }),
                            Transform::identity(),
                        ))
                    }
                } else {
                    None
                }
            })
        })
        .chain(big)
        .collect();

    WorldInfo {
        objects,
        lights: vec![],
        background: Box::new(Sky::default()),
        camera: Camera::new(CameraInfo {
            aspect_ratio: 1.0,
            fov: 20.,
            origin: Point3::new(13.0, 2.0, 3.0),
            look_at: Point3::new(0.0, 0.0, 0.0),
            up_vector: Vector3::unit_y(),
            aperture: 0.0005,
            focus_distance: 10.,
            start_time: 0.0,
            end_time: 0.,
        }),
        sun: None,
    }
}

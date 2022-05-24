use super::{
    super::{IMAGE_HEIGHT, IMAGE_WIDTH},
    Camera, Dielectric, Hittable, Lambertian, Metal, MovingSphere, Sky, SolidColor, Sphere, World,
};
use crate::prelude::*;
use cgmath::{prelude::*, Point3, Vector3};
use std::{cell::RefCell, rc::Rc};
#[allow(dead_code)]
pub fn random_scene() -> (World, Camera) {
    let big: [Rc<dyn Hittable>; 4] = [
        Rc::new(Sphere {
            radius: 1000.0,
            origin: Point3::new(0.0, -1000.0, 1000.0),
            material: Rc::new(RefCell::new(Lambertian {
                albedo: Box::new(SolidColor {
                    color: RgbColor {
                        red: 0.5,
                        green: 0.5,
                        blue: 0.5,
                    },
                }),
            })),
        }),
        Rc::new(Sphere {
            radius: 1.0,
            origin: Point3::new(0.0, 1.0, 0.0),
            material: Rc::new(RefCell::new(Dielectric {
                index_refraction: 1.5,
                color: RgbColor::new(1.0, 1.0, 1.0),
            })),
        }),
        Rc::new(Sphere {
            radius: 1.0,
            origin: Point3::new(-4.0, 1.0, 0.0),
            material: Rc::new(RefCell::new(Lambertian {
                albedo: Box::new(SolidColor {
                    color: RgbColor::new(0.4, 0.2, 0.1),
                }),
            })),
        }),
        Rc::new(Sphere {
            radius: 1.0,
            origin: Point3::new(4.0, 1.0, 0.0),
            material: Rc::new(RefCell::new(Metal {
                albedo: Box::new(SolidColor {
                    color: RgbColor::new(0.4, 0.2, 0.1),
                }),
                fuzz: 0.0,
            })),
        }),
    ];
    let spheres = (-11..11)
        .flat_map(|a| {
            (-11..11).filter_map::<Rc<dyn Hittable>, _>(move |b| {
                let choose_mat = rand::random::<f32>();
                let center = Point3::new(
                    a as f32 + 0.9 * rand::random::<f32>(),
                    0.2,
                    b as f32 + 0.9 * rand::random::<f32>(),
                );
                let check = center - Point3::new(4.0, 0.2, 0.0);
                if check.dot(check).sqrt() > 0.9 {
                    if choose_mat < 0.8 {
                        Some(Rc::new(MovingSphere {
                            radius: 0.2,
                            center_0: center,
                            center_1: center + Vector3::new(0.0, rand_f32(0.0, 0.5), 0.0),
                            time_0: 0.0,
                            time_1: 1.0,
                            material: Rc::new(RefCell::new(Lambertian {
                                albedo: Box::new(SolidColor {
                                    color: RgbColor::random(),
                                }),
                            })),
                        }))
                    } else if choose_mat < 0.95 {
                        Some(Rc::new(Sphere {
                            radius: 0.2,
                            origin: center,
                            material: Rc::new(RefCell::new(Metal {
                                albedo: Box::new(SolidColor {
                                    color: RgbColor::random(),
                                }),
                                fuzz: rand::random::<f32>() * 0.5 + 0.5,
                            })),
                        }))
                    } else {
                        Some(Rc::new(Sphere {
                            radius: 0.2,
                            origin: center,
                            material: Rc::new(RefCell::new(Dielectric {
                                color: RgbColor::new(1.0, 1.0, 1.0),
                                index_refraction: 1.5,
                            })),
                        }))
                    }
                } else {
                    None
                }
            })
        })
        .chain(big)
        .collect();
    (
        World {
            spheres,
            lights: vec![],
            background: Box::new(Sky {}),
        },
        Camera::new(
            IMAGE_WIDTH as f32 / IMAGE_HEIGHT as f32,
            20.0,
            Point3::new(13.0, 2.0, 3.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            0.0005,
            10.0,
            0.0,
            1.0,
        ),
    )
}

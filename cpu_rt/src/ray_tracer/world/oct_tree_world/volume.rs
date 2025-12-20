use super::{
    Camera, CameraInfo, ConstantColor, DiffuseLight, Object, OctTree, RayScalar, RgbColor,
    SolidColor, Sphere, Transform, Voxel, WorldInfo,
};
use crate::ray_tracer::{
    background::Sky,
    hittable::{SolidVoxel, VolumeEdgeEffect, VolumeVoxel},
};
use cgmath::{prelude::*, Point3, Vector3};

pub fn oct_tree_volume() -> WorldInfo {
    let look_at = Point3::<RayScalar>::new(5.0, 5.0, 5.0);

    let origin = Point3::<RayScalar>::new(-20.0, 5.0, -20.0);
    let fov = 40.0;
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    let light = Box::new(DiffuseLight {
        emit: Box::new(SolidColor {
            color: 500.0 * RgbColor::WHITE,
        }),
    });
    //let solid = Box::new();
    let light = Object::new(
        Box::new(Sphere {
            radius: 1.0,
            origin: Point3::new(20.0, 5.0, 20.0),
            material: light,
        }),
        Transform::identity(),
    );
    let mut tree = OctTree::<Voxel>::empty();
    for x in 0..10 {
        for y in 1..9 {
            for z in 0..10 {
                tree.set(
                    Point3 { x, y, z },
                    Voxel::Volume(VolumeVoxel {
                        density: 0.3,
                        color: RgbColor::new(0.5, 0.05, 0.5),
                        edge_effect: VolumeEdgeEffect::None,
                    }),
                );
            }
        }
    }
    for x in 3..6 {
        for y in 3..6 {
            for z in 3..6 {
                tree.set(
                    Point3 { x, y, z },
                    Voxel::Solid(SolidVoxel::Lambertian {
                        albedo: RgbColor::new(0.65, 0.05, 0.05),
                    }),
                )
            }
        }
    }

    tree.set(
        Point3::new(0, 0, 0),
        Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor::new(0.65, 0.05, 0.05),
        }),
    );
    tree.set(
        Point3::new(0, 1, 0),
        Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor::new(0.65, 0.05, 0.05),
        }),
    );
    tree.set(
        Point3::new(5, 5, 5),
        Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor::new(0.65, 0.05, 0.05),
        }),
    );

    WorldInfo {
        objects: vec![
            Object::new(Box::new(tree), Transform::identity()),
            light.clone(),
        ],
        lights: vec![light],

        background: Box::new(ConstantColor {
            color: 0.1 * RgbColor::new(1.0, 1.0, 1.0),
        }),
        camera: Camera::new(CameraInfo {
            aspect_ratio: 1.0,
            fov,
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
pub fn oct_tree_volume_two_density() -> WorldInfo {
    let look_at = Point3::<RayScalar>::new(5.0, 5.0, 5.0);

    let origin = Point3::<RayScalar>::new(-20.0, 5.0, -20.0);
    let fov = 40.0;
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    let light = Box::new(DiffuseLight {
        emit: Box::new(SolidColor {
            color: 500.0 * RgbColor::WHITE,
        }),
    });
    //let solid = Box::new();
    let light = Object::new(
        Box::new(Sphere {
            radius: 1.0,
            origin: Point3::new(50.0, 0.0, 20.0),
            material: light,
        }),
        Transform::identity(),
    );
    let mut tree = OctTree::<Voxel>::empty();
    for x in 0..10 {
        for y in 1..9 {
            for z in 0..10 {
                tree.set(
                    Point3 { x, y, z },
                    Voxel::Volume(VolumeVoxel {
                        density: if y < 5 { 0.3 } else { 0.6 },
                        color: RgbColor::new(0.5, 0.05, 0.5),
                        edge_effect: VolumeEdgeEffect::None,
                    }),
                );
            }
        }
    }
    for x in 3..6 {
        for y in 3..6 {
            for z in 3..6 {
                tree.set(
                    Point3 { x, y, z },
                    Voxel::Solid(SolidVoxel::Lambertian {
                        albedo: RgbColor::new(0.65, 0.05, 0.05),
                    }),
                )
            }
        }
    }

    tree.set(
        Point3::new(0, 0, 0),
        Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor::new(0.65, 0.05, 0.05),
        }),
    );
    tree.set(
        Point3::new(0, 1, 0),
        Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor::new(0.65, 0.05, 0.05),
        }),
    );
    tree.set(
        Point3::new(5, 5, 5),
        Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor::new(0.65, 0.05, 0.05),
        }),
    );

    WorldInfo {
        objects: vec![
            Object::new(Box::new(tree), Transform::identity()),
            light.clone(),
        ],
        lights: vec![light],

        background: Box::new(ConstantColor {
            color: 0.1 * RgbColor::new(1.0, 1.0, 1.0),
        }),
        camera: Camera::new(CameraInfo {
            aspect_ratio: 1.0,
            fov,
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
pub fn oct_tree_volume_many_density() -> WorldInfo {
    use rand::{rngs::StdRng, Rng, SeedableRng};
    let look_at = Point3::<RayScalar>::new(5.0, 5.0, 5.0);

    let origin = Point3::<RayScalar>::new(-20.0, 5.0, -20.0);
    let fov = 40.0;
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    let light = Box::new(DiffuseLight {
        emit: Box::new(SolidColor {
            color: 500.0 * RgbColor::WHITE,
        }),
    });

    let light = Object::new(
        Box::new(Sphere {
            radius: 1.0,
            origin: Point3::new(50.0, 0.0, 20.0),
            material: light,
        }),
        Transform::identity(),
    );
    let mut tree = OctTree::<Voxel>::empty();

    let mut rng = StdRng::from_seed([4u8; 32]);
    for x in 0..10 {
        for y in 1..9 {
            for z in 0..10 {
                tree.set(
                    Point3 { x, y, z },
                    Voxel::Volume(VolumeVoxel {
                        density: rng.gen_range(0.3..0.6),
                        color: RgbColor::new(0.5, 0.05, 0.5),
                        edge_effect: VolumeEdgeEffect::None,
                    }),
                );
            }
        }
    }
    for x in 3..6 {
        for y in 3..6 {
            for z in 3..6 {
                tree.set(
                    Point3 { x, y, z },
                    Voxel::Solid(SolidVoxel::Lambertian {
                        albedo: RgbColor::new(0.65, 0.05, 0.05),
                    }),
                )
            }
        }
    }

    tree.set(
        Point3::new(0, 0, 0),
        Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor::new(0.65, 0.05, 0.05),
        }),
    );
    tree.set(
        Point3::new(0, 1, 0),
        Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor::new(0.65, 0.05, 0.05),
        }),
    );
    tree.set(
        Point3::new(5, 5, 5),
        Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor::new(0.65, 0.05, 0.05),
        }),
    );

    WorldInfo {
        objects: vec![
            Object::new(Box::new(tree), Transform::identity()),
            light.clone(),
        ],
        lights: vec![light],

        background: Box::new(ConstantColor {
            color: 0.1 * RgbColor::new(1.0, 1.0, 1.0),
        }),
        camera: Camera::new(CameraInfo {
            aspect_ratio: 1.0,
            fov,
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
pub fn oct_tree_volume_lambertian() -> WorldInfo {
    let look_at = Point3::<RayScalar>::new(5.0, 5.0, 5.0);

    let origin = Point3::<RayScalar>::new(-20.0, 5.0, -20.0);
    let fov = 40.0;
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    let light = Box::new(DiffuseLight {
        emit: Box::new(SolidColor {
            color: 500.0 * RgbColor::WHITE,
        }),
    });

    let light = Object::new(
        Box::new(Sphere {
            radius: 1.0,
            origin: Point3::new(50.0, 0.0, 20.0),
            material: light,
        }),
        Transform::identity(),
    );
    let mut tree = OctTree::<Voxel>::empty();

    for x in 0..10 {
        for y in 1..9 {
            for z in 0..10 {
                tree.set(
                    Point3 { x, y, z },
                    Voxel::Volume(VolumeVoxel {
                        density: 0.3,
                        color: RgbColor::new(0.5, 0.05, 0.5),
                        edge_effect: VolumeEdgeEffect::Solid {
                            hit_probability: 0.6,
                            solid_material: SolidVoxel::Lambertian {
                                albedo: RgbColor::new(0.5, 0.05, 0.5),
                            },
                        },
                    }),
                );
            }
        }
    }
    for x in 3..6 {
        for y in 3..6 {
            for z in 3..6 {
                tree.set(
                    Point3 { x, y, z },
                    Voxel::Solid(SolidVoxel::Lambertian {
                        albedo: RgbColor::new(0.65, 0.05, 0.05),
                    }),
                )
            }
        }
    }

    tree.set(
        Point3::new(0, 0, 0),
        Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor::new(0.65, 0.05, 0.05),
        }),
    );
    tree.set(
        Point3::new(0, 1, 0),
        Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor::new(0.65, 0.05, 0.05),
        }),
    );
    tree.set(
        Point3::new(5, 5, 5),
        Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor::new(0.65, 0.05, 0.05),
        }),
    );

    WorldInfo {
        objects: vec![
            Object::new(Box::new(tree), Transform::identity()),
            light.clone(),
        ],
        lights: vec![light],

        background: Box::new(ConstantColor {
            color: 0.1 * RgbColor::new(1.0, 1.0, 1.0),
        }),
        camera: Camera::new(CameraInfo {
            aspect_ratio: 1.0,
            fov,
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
pub fn oct_tree_volume_metal() -> WorldInfo {
    let look_at = Point3::<RayScalar>::new(5.0, 5.0, 5.0);

    let origin = Point3::<RayScalar>::new(-20.0, 5.0, -20.0);
    let fov = 40.0;
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    let light = Object::new(
        Box::new(Sphere {
            radius: 1.0,
            origin: Point3::new(0.0, 100.0, 200.0),
            material: Box::new(DiffuseLight {
                emit: Box::new(SolidColor {
                    color: 1000.0 * RgbColor::WHITE,
                }),
            }),
        }),
        Transform::identity(),
    );
    let mut tree = OctTree::<Voxel>::empty();

    for x in 0..10 {
        for y in 1..9 {
            for z in 0..10 {
                tree.set(
                    Point3 { x, y, z },
                    Voxel::Volume(VolumeVoxel {
                        density: 0.3,
                        color: RgbColor::new(0.5, 0.05, 0.5),
                        edge_effect: VolumeEdgeEffect::Solid {
                            hit_probability: 0.6,
                            solid_material: SolidVoxel::Reflect {
                                albedo: RgbColor::new(0.5, 0.05, 0.5),
                                fuzz: 0.3,
                            },
                        },
                    }),
                );
            }
        }
    }
    for x in 3..6 {
        for y in 3..6 {
            for z in 3..6 {
                tree.set(
                    Point3 { x, y, z },
                    Voxel::Solid(SolidVoxel::Lambertian {
                        albedo: RgbColor::new(0.65, 0.05, 0.05),
                    }),
                )
            }
        }
    }

    tree.set(
        Point3::new(0, 0, 0),
        Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor::new(0.65, 0.05, 0.05),
        }),
    );
    tree.set(
        Point3::new(0, 1, 0),
        Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor::new(0.65, 0.05, 0.05),
        }),
    );
    tree.set(
        Point3::new(5, 5, 5),
        Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor::new(0.65, 0.05, 0.05),
        }),
    );
    for x in 0..40 {
        for z in 0..40 {
            tree.set(
                Point3 { x, y: 0, z },
                Voxel::Solid(SolidVoxel::Lambertian {
                    albedo: RgbColor::new(0.9, 0.9, 0.9),
                }),
            )
        }
    }
    WorldInfo {
        objects: vec![
            Object::new(Box::new(tree), Transform::identity()),
            light.clone(),
        ],
        lights: vec![light],

        background: Box::new(ConstantColor {
            color: 0.1 * RgbColor::new(1.0, 1.0, 1.0),
        }),
        camera: Camera::new(CameraInfo {
            aspect_ratio: 1.0,
            fov,
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
pub fn oct_tree_volume_ice() -> WorldInfo {
    let look_at = Point3::<RayScalar>::new(5.0, 5.0, 5.0);

    let origin = Point3::<RayScalar>::new(-20.0, 20.0, -20.0);
    let fov = 40.0;
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    let light = Object::new(
        Box::new(Sphere {
            radius: 1.0,
            origin: Point3::new(0.0, 100.0, 200.0),
            material: Box::new(DiffuseLight {
                emit: Box::new(SolidColor {
                    color: 500000.0 * RgbColor::WHITE,
                }),
            }),
        }),
        Transform::identity(),
    );
    let mut tree = OctTree::<Voxel>::empty();

    for x in 0..10 {
        for y in 1..9 {
            for z in 0..10 {
                let offset = Vector3::new(10, 0, 10);
                let ice_color = RgbColor::from_color_hex("#06068dff");
                let snow_color = RgbColor::from_color_hex("#ffffffff");
                let value = if y >= 8 {
                    Voxel::Volume(VolumeVoxel {
                        density: 0.001,
                        color: snow_color,
                        edge_effect: VolumeEdgeEffect::Solid {
                            hit_probability: 1.,
                            solid_material: SolidVoxel::Lambertian { albedo: snow_color },
                        },
                    })
                } else {
                    Voxel::Volume(VolumeVoxel {
                        density: 0.5,
                        color: ice_color,
                        edge_effect: VolumeEdgeEffect::Solid {
                            hit_probability: 0.7,
                            solid_material: SolidVoxel::Reflect {
                                albedo: ice_color,
                                fuzz: 0.1,
                            },
                        },
                    })
                };
                tree.set(Point3 { x, y, z } + offset, value);
            }
        }
    }

    tree.set(
        Point3::new(0, 0, 0),
        Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor::new(0.65, 0.05, 0.05),
        }),
    );
    tree.set(
        Point3::new(0, 1, 0),
        Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor::new(0.65, 0.05, 0.05),
        }),
    );

    for x in 0..40 {
        for z in 0..40 {
            let v = (x + z) & 0x1;
            let color = match v {
                0 => RgbColor::from_color_hex("#ffffffff"),
                1 => RgbColor::from_color_hex("#fc0202ff"),
                _ => panic!("can never get here"),
            };
            tree.set(
                Point3 { x, y: 0, z },
                Voxel::Solid(SolidVoxel::Lambertian { albedo: color }),
            )
        }
    }
    WorldInfo {
        objects: vec![
            Object::new(Box::new(tree), Transform::identity()),
            light.clone(),
        ],
        lights: vec![light],

        background: Box::new(Sky { intensity: 0.6 }),
        camera: Camera::new(CameraInfo {
            aspect_ratio: 1.0,
            fov,
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

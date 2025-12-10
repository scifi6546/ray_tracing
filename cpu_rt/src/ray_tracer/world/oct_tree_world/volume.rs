use super::{
    Camera, CameraInfo, ConstantColor, DiffuseLight, Object, OctTree, RayScalar, RgbColor,
    SolidColor, Sphere, Transform, VoxelMaterial, WorldInfo,
};
use crate::ray_tracer::hittable::VolumeEdgeEffect;
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
    let mut tree = OctTree::<VoxelMaterial>::empty();
    for x in 0..10 {
        for y in 1..9 {
            for z in 0..10 {
                tree.set(
                    Point3 { x, y, z },
                    VoxelMaterial::Volume {
                        density: 0.3,
                        color: RgbColor::new(0.5, 0.05, 0.5),
                        edge_effect: VolumeEdgeEffect::None,
                    },
                );
            }
        }
    }
    for x in 3..6 {
        for y in 3..6 {
            for z in 3..6 {
                tree.set(
                    Point3 { x, y, z },
                    VoxelMaterial::Solid {
                        color: RgbColor::new(0.65, 0.05, 0.05),
                    },
                )
            }
        }
    }

    tree.set(
        Point3::new(0, 0, 0),
        VoxelMaterial::Solid {
            color: RgbColor::new(0.65, 0.05, 0.05),
        },
    );
    tree.set(
        Point3::new(0, 1, 0),
        VoxelMaterial::Solid {
            color: RgbColor::new(0.65, 0.05, 0.05),
        },
    );
    tree.set(
        Point3::new(5, 5, 5),
        VoxelMaterial::Solid {
            color: RgbColor::new(0.65, 0.05, 0.05),
        },
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
    let mut tree = OctTree::<VoxelMaterial>::empty();
    for x in 0..10 {
        for y in 1..9 {
            for z in 0..10 {
                tree.set(
                    Point3 { x, y, z },
                    VoxelMaterial::Volume {
                        density: if y < 5 { 0.3 } else { 0.6 },
                        color: RgbColor::new(0.5, 0.05, 0.5),
                        edge_effect: VolumeEdgeEffect::None,
                    },
                );
            }
        }
    }
    for x in 3..6 {
        for y in 3..6 {
            for z in 3..6 {
                tree.set(
                    Point3 { x, y, z },
                    VoxelMaterial::Solid {
                        color: RgbColor::new(0.65, 0.05, 0.05),
                    },
                )
            }
        }
    }

    tree.set(
        Point3::new(0, 0, 0),
        VoxelMaterial::Solid {
            color: RgbColor::new(0.65, 0.05, 0.05),
        },
    );
    tree.set(
        Point3::new(0, 1, 0),
        VoxelMaterial::Solid {
            color: RgbColor::new(0.65, 0.05, 0.05),
        },
    );
    tree.set(
        Point3::new(5, 5, 5),
        VoxelMaterial::Solid {
            color: RgbColor::new(0.65, 0.05, 0.05),
        },
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
    let mut tree = OctTree::<VoxelMaterial>::empty();

    let mut rng = StdRng::from_seed([4u8; 32]);
    for x in 0..10 {
        for y in 1..9 {
            for z in 0..10 {
                tree.set(
                    Point3 { x, y, z },
                    VoxelMaterial::Volume {
                        density: rng.gen_range(0.3..0.6),
                        color: RgbColor::new(0.5, 0.05, 0.5),
                        edge_effect: VolumeEdgeEffect::None,
                    },
                );
            }
        }
    }
    for x in 3..6 {
        for y in 3..6 {
            for z in 3..6 {
                tree.set(
                    Point3 { x, y, z },
                    VoxelMaterial::Solid {
                        color: RgbColor::new(0.65, 0.05, 0.05),
                    },
                )
            }
        }
    }

    tree.set(
        Point3::new(0, 0, 0),
        VoxelMaterial::Solid {
            color: RgbColor::new(0.65, 0.05, 0.05),
        },
    );
    tree.set(
        Point3::new(0, 1, 0),
        VoxelMaterial::Solid {
            color: RgbColor::new(0.65, 0.05, 0.05),
        },
    );
    tree.set(
        Point3::new(5, 5, 5),
        VoxelMaterial::Solid {
            color: RgbColor::new(0.65, 0.05, 0.05),
        },
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
    let mut tree = OctTree::<VoxelMaterial>::empty();

    for x in 0..10 {
        for y in 1..9 {
            for z in 0..10 {
                tree.set(
                    Point3 { x, y, z },
                    VoxelMaterial::Volume {
                        density: 0.3,
                        color: RgbColor::new(0.5, 0.05, 0.5),
                        edge_effect: VolumeEdgeEffect::Lambertian {
                            hit_probability: 0.6,
                        },
                    },
                );
            }
        }
    }
    for x in 3..6 {
        for y in 3..6 {
            for z in 3..6 {
                tree.set(
                    Point3 { x, y, z },
                    VoxelMaterial::Solid {
                        color: RgbColor::new(0.65, 0.05, 0.05),
                    },
                )
            }
        }
    }

    tree.set(
        Point3::new(0, 0, 0),
        VoxelMaterial::Solid {
            color: RgbColor::new(0.65, 0.05, 0.05),
        },
    );
    tree.set(
        Point3::new(0, 1, 0),
        VoxelMaterial::Solid {
            color: RgbColor::new(0.65, 0.05, 0.05),
        },
    );
    tree.set(
        Point3::new(5, 5, 5),
        VoxelMaterial::Solid {
            color: RgbColor::new(0.65, 0.05, 0.05),
        },
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

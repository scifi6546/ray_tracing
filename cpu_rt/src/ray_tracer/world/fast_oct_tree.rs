use crate::prelude::iter_box;

use super::{
    world_prelude::{
        Camera, CameraInfo, ConstantColor, DiffuseLight, FastOctTree, Object, RayScalar, RgbColor,
        Sky, SolidColor, SolidVoxel, Sphere, Transform, Voxel,
    },
    WorldInfo,
};
use cgmath::{prelude::*, Point2, Point3, Vector3};

pub fn fast_oct_tree_sphere() -> WorldInfo {
    //let origin = Point3::<RayScalar>::new(1.0, 1.0, 1.0);
    let origin = Point3::<RayScalar>::new(-100.0, 10., 100.0);
    let look_at = Point3::<RayScalar>::new(10., 10., 10.);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };
    let top_light = Object::new(
        Box::new(Sphere {
            radius: 10.0,
            origin: Point3::new(-320.0, 100.0, -100.0),
            material: Box::new(DiffuseLight {
                emit: Box::new(SolidColor {
                    color: 400.0 * RgbColor::WHITE,
                }),
            }),
        }),
        Transform::identity(),
    );
    WorldInfo {
        objects: vec![
            Object::new(
                Box::new(FastOctTree::<Voxel>::sphere(
                    10,
                    Voxel::Solid(SolidVoxel::Lambertian {
                        albedo: RgbColor::new(0.5, 0.5, 0.5),
                    }),
                )),
                Transform::identity(),
            ),
            top_light.clone(),
        ],
        lights: vec![top_light],
        background: Box::new(ConstantColor {
            color: 0.1 * RgbColor::WHITE,
        }),
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
pub fn sinnoh() -> WorldInfo {
    let world =
        FastOctTree::load_map("./voxel_assets/sinnoh/twinleaf.yml").expect("failed to load world");
    let tile_size_x = 16;
    let tile_size_z = 16;

    let block_x = tile_size_x * 32;

    let block_z = tile_size_z * 32;

    let fov = 40.0;

    let look_at = Point3::new(block_x as RayScalar / 2.0, 10.0, block_z as RayScalar / 2.0);
    //let look_at = Point3::new(0.0, 0.0, 0.0);
    let origin = Point3::<RayScalar>::new(-500.0, 300.0, block_z as RayScalar / 2.0);

    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    WorldInfo {
        objects: vec![Object::new(Box::new(world), Transform::identity())],
        lights: vec![],
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
pub fn explosion() -> WorldInfo {
    let look_at = Point3::<RayScalar>::new(50.0, 10.0, 50.0);

    let origin = Point3::<RayScalar>::new(-20.0, 50.0, -20.0);

    let fov = 40.0;
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    let light = Box::new(DiffuseLight {
        emit: Box::new(SolidColor {
            color: 20000.0 * RgbColor::new(252.0 / 255.0, 79.0 / 255.0, 5.0 / 255.0),
        }),
    });
    let lava_light = Object::new(
        Box::new(Sphere {
            radius: 3.0,
            origin: Point3::new(50.0, 28.0, 50.0),
            material: light.clone(),
        }),
        Transform::identity(),
    );

    const MAX_Y: i32 = 20;
    fn height(x: isize, z: isize) -> isize {
        let center = Point2::new(50.0, 50.0);
        let radius = center.distance(Point2::new(x as f32, z as f32));
        let h = (radius / 10.0).cos() * 10.0 + 15.0;
        h.max(0.0).min((MAX_Y - 1) as f32) as isize
    }
    let mut tree = FastOctTree::<Voxel>::new();
    for x in 0..100 {
        for z in 0..100 {
            let h = height(x, z);
            for y in 0..=h {
                let albedo = if y < 9 {
                    RgbColor::new(0.65, 0.8, 0.05)
                } else {
                    RgbColor::new(0.65, 0.05, 0.05)
                };
                tree.set(
                    Voxel::Solid(SolidVoxel::Lambertian { albedo }),
                    Point3::new(x as u32, y as u32, z as u32),
                );
            }
        }
    }
    WorldInfo {
        objects: vec![
            Object::new(Box::new(tree), Transform::identity()),
            lava_light.clone(),
        ],
        lights: vec![lava_light],
        background: Box::new(Sky { intensity: 0.1 }),
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
pub fn cube_world() -> WorldInfo {
    let look_at = Point3::<RayScalar>::new(0.0, 5.0, 5.0);

    let origin = Point3::<RayScalar>::new(-20.0, 5.0, -20.0);

    let fov = 40.0;
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    let light = Box::new(DiffuseLight {
        emit: Box::new(SolidColor {
            color: 20000.0 * RgbColor::WHITE,
        }),
    });
    let light = Object::new(
        Box::new(Sphere {
            radius: 1.0,
            origin: Point3::new(10.0, 10.0, -10.0),
            material: light,
        }),
        Transform::identity(),
    );
    let mut tree = FastOctTree::<Voxel>::new();
    for i in 3..6 {
        for j in 3..6 {
            for k in 3..6 {
                tree.set(
                    Voxel::Solid(SolidVoxel::Lambertian {
                        albedo: RgbColor::new(0.65, 0.05, 0.05),
                    }),
                    Point3::new(i, j, k),
                )
            }
        }
    }
    for pos in [
        Point3::new(0, 0, 0),
        Point3::new(0, 1, 0),
        Point3::new(5, 5, 5),
    ] {
        tree.set(
            Voxel::Solid(SolidVoxel::Lambertian {
                albedo: RgbColor::new(0.65, 0.05, 0.05),
            }),
            pos,
        )
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
pub fn volcano() -> WorldInfo {
    fn cone(position: Point3<RayScalar>, center: Point3<RayScalar>, slope: RayScalar) -> bool {
        let desired_radius = (position.y - center.y) / slope;
        let desired_radius_squared = if desired_radius > 0. {
            desired_radius * desired_radius
        } else {
            0.
        };
        let distance_squared = (position.x - center.x).powi(2) + (position.z - center.z).powi(2);
        distance_squared < desired_radius_squared
    }
    let origin = Point3::<RayScalar>::new(1000., 500.0, -300.0);
    let look_at = Point3::<RayScalar>::new(500., 200.0, 500.0);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };
    let top_light = Object::new(
        Box::new(Sphere {
            radius: 10.0,
            origin: Point3::new(-100., 50., -100.0),
            material: Box::new(DiffuseLight {
                emit: Box::new(SolidColor {
                    color: RgbColor::WHITE * 100.,
                }),
            }),
        }),
        Transform::identity(),
    );
    let mut tree = FastOctTree::new();
    for position in iter_box(Point3::new(1000, 1000, 1000)) {
        let position_float = position.map(|v| v as RayScalar);
        let mountain_cone = cone(position_float, Point3::new(500., 300., 500.), -1.);
        let crater = cone(position_float, Point3::new(500., 125., 500.), 0.3);
        if mountain_cone && (!crater || position.y < 150) {
            let value = if crater {
                Voxel::Solid(SolidVoxel::Lambertian {
                    albedo: RgbColor::from_color_hex("#ffbb00") * 80.,
                })
            } else {
                Voxel::Solid(SolidVoxel::Lambertian {
                    albedo: RgbColor::from_color_hex("#5f1515"),
                })
            };
            tree.set(value, position);
        }
    }
    WorldInfo {
        objects: vec![
            Object::new(Box::new(tree), Transform::identity()),
            top_light.clone(),
        ],
        lights: vec![top_light],
        background: Box::new(ConstantColor {
            color: RgbColor::new(0.1, 0.1, 0.1),
        }),
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

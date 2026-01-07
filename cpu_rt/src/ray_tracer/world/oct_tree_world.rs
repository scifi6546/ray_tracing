pub mod compare_voxel_world;
mod load_model;
pub mod metal;
pub mod volume;
use super::{
    Camera, CameraInfo, ConstantColor, DiffuseLight, OctTree, Sky, SolidColor, Sphere, Sun, SunSky,
    Transform, Voxel, WorldInfo,
};
pub use load_model::load_voxel_model;

use crate::{
    prelude::*,
    ray_tracer::hittable::{Object, SolidVoxel},
};
use cgmath::{num_traits::FloatConst, prelude::*, Point2, Point3, Vector3};

pub fn basic_sphere() -> WorldInfo {
    let origin = Point3::<RayScalar>::new(1.0, 1.0, 1.0);
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
                Box::new(OctTree::sphere(
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

pub fn temple() -> WorldInfo {
    fn make_brick(size: Vector3<u32>, material: Voxel) -> OctTree<Voxel> {
        OctTree::rectangle(size, material)
    }
    let sun = Sun {
        phi: RayScalar::FRAC_PI_4(),
        theta: 0.1,
        radius: 1.0,
    };
    let sun_sky = SunSky::new(sun, 0.1, 10.);
    let origin = Point3::<RayScalar>::new(-100.0, 100.0, -800.0);
    let look_at = Point3::<RayScalar>::new(64.0, 64.0, 64.0);
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
                    color: RgbColor::new(15.0, 15.0, 15.0),
                }),
            }),
        }),
        Transform::identity(),
    );
    let mut temple = OctTree::rectangle(
        Vector3::new(1000, 10, 1000),
        Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor::new(0.0, 0.1, 0.0),
        }),
    );
    let mut temple_wall = OctTree::empty();
    let box_x_size = 40;
    let box_y_size = 20;
    let box_z_size = 20;
    let mortar = 1;
    for x in 0..5 {
        for y in 0..10 {
            let brick = make_brick(
                Vector3::new(box_x_size, box_y_size, box_z_size),
                Voxel::Solid(SolidVoxel::Lambertian {
                    albedo: RgbColor::new(0.3, 0.3, 0.3),
                }),
            );
            temple_wall = temple_wall.combine(
                &brick,
                Point3::new(
                    (x * (box_x_size + mortar) + (y % 2) * (box_x_size / 2)) as i32,
                    (y * (box_y_size + mortar)) as i32,
                    0,
                ),
            );
        }
    }
    temple = temple.combine(&temple_wall, Point3::new(0, 0, 800));
    WorldInfo {
        objects: vec![
            Object::new(Box::new(temple), Transform::identity()),
            top_light.clone(),
        ],
        lights: vec![top_light],
        background: Box::new(sun_sky),
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
pub fn temple_below() -> WorldInfo {
    let origin = Point3::<RayScalar>::new(-100.0, -100.0, -800.0);
    let look_at = Point3::<RayScalar>::new(64.0, 64.0, 64.0);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };
    let light_color = 100.0 * RgbColor::new(30.0, 30.0, 30.0);
    let top_light = Object::new(
        Box::new(Sphere {
            radius: 10.0,
            origin: Point3::new(-320.0, 100.0, -100.0),
            material: Box::new(DiffuseLight {
                emit: Box::new(SolidColor { color: light_color }),
            }),
        }),
        Transform::identity(),
    );
    let temple = OctTree::rectangle(
        Vector3::new(5, 100, 100),
        Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor::new(0.5, 0.5, 0.5),
        }),
    )
    .combine(
        &OctTree::rectangle(
            Vector3::new(100, 5, 100),
            Voxel::Solid(SolidVoxel::Lambertian {
                albedo: RgbColor::new(0.5, 0.5, 0.5),
            }),
        ),
        Point3::new(0, 0, 0),
    );
    let bg_color = RgbColor::new(0.02, 0.02, 0.02);
    WorldInfo {
        objects: vec![
            Object::new(Box::new(temple), Transform::identity()),
            top_light.clone(),
        ],
        lights: vec![top_light],
        background: Box::new(ConstantColor { color: bg_color }),
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
pub fn cube() -> WorldInfo {
    let origin = Point3::<RayScalar>::new(-100.0, 100.0, -800.0);
    let look_at = Point3::<RayScalar>::new(64.0, 64.0, 64.0);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };
    let top_light = Object::new(
        Box::new(Sphere {
            radius: 10.0,
            origin: Point3::new(0., 0., -100.0),
            material: Box::new(DiffuseLight {
                emit: Box::new(SolidColor {
                    color: RgbColor::WHITE * 100.,
                }),
            }),
        }),
        Transform::identity(),
    );
    let temple = OctTree::cube_pow(
        4,
        Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor::new(0.5, 0.5, 0.5),
        }),
    );
    WorldInfo {
        objects: vec![
            Object::new(Box::new(temple), Transform::identity()),
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

pub fn cube_back() -> WorldInfo {
    let origin = Point3::<RayScalar>::new(100.0, 100.0, 800.0);
    let look_at = Point3::<RayScalar>::new(64.0, 64.0, 64.0);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };
    let top_light = Object::new(
        Box::new(Sphere {
            radius: 10.0,
            origin: Point3::new(0., 0., -100.0),
            material: Box::new(DiffuseLight {
                emit: Box::new(SolidColor {
                    color: RgbColor::WHITE * 100.,
                }),
            }),
        }),
        Transform::identity(),
    );
    let cube = OctTree::cube_pow(
        4,
        Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor::new(0.5, 0.5, 0.5),
        }),
    );
    WorldInfo {
        objects: vec![
            Object::new(Box::new(cube), Transform::identity()),
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
    let mut tree = OctTree::<Voxel>::empty();
    for i in 3..6 {
        for j in 3..6 {
            for k in 3..6 {
                tree.set(
                    Point3::new(i, j, k),
                    Voxel::Solid(SolidVoxel::Lambertian {
                        albedo: RgbColor::new(0.65, 0.05, 0.05),
                    }),
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
            pos,
            Voxel::Solid(SolidVoxel::Lambertian {
                albedo: RgbColor::new(0.65, 0.05, 0.05),
            }),
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
    let mut tree = OctTree::<Voxel>::empty();
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
                    Point3::new(x as u32, y as u32, z as u32),
                    Voxel::Solid(SolidVoxel::Lambertian { albedo }),
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

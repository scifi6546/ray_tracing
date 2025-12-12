pub mod compare_voxel_world;
pub mod volume;
use super::{
    Camera, CameraInfo, ConstantColor, DiffuseLight, OctTree, SolidColor, Sphere, Sun, SunSky,
    Transform, Voxel, WorldInfo,
};
use crate::prelude::*;
use crate::ray_tracer::hittable::{Object, SolidVoxel};
use cgmath::{num_traits::FloatConst, prelude::*, Point3, Vector3};

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
                    Voxel::Solid(SolidVoxel {
                        color: RgbColor::new(0.5, 0.5, 0.5),
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
        Voxel::Solid(SolidVoxel {
            color: RgbColor::new(0.0, 0.1, 0.0),
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
                Voxel::Solid(SolidVoxel {
                    color: RgbColor::new(0.3, 0.3, 0.3),
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
        Voxel::Solid(SolidVoxel {
            color: RgbColor::new(0.5, 0.5, 0.5),
        }),
    )
    .combine(
        &OctTree::rectangle(
            Vector3::new(100, 5, 100),
            Voxel::Solid(SolidVoxel {
                color: RgbColor::new(0.5, 0.5, 0.5),
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
        Voxel::Solid(SolidVoxel {
            color: RgbColor::new(0.5, 0.5, 0.5),
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
        Voxel::Solid(SolidVoxel {
            color: RgbColor::new(0.5, 0.5, 0.5),
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

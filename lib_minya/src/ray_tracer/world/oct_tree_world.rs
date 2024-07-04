use super::{
    Camera, ConstantColor, DiffuseLight, OctTree, SolidColor, Sphere, Transform, VoxelMaterial,
    WorldInfo,
};
use crate::prelude::*;
use crate::ray_tracer::hittable::Object;
use cgmath::{prelude::*, Point3, Vector3};

pub fn basic_sphere() -> WorldInfo {
    let origin = Point3::new(-100.0f32, 100.0, -800.0);
    let look_at = Point3::new(64.0f32, 64.0, 64.0);
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
    WorldInfo {
        objects: vec![
            Object::new(
                Box::new(OctTree::sphere(10, VoxelMaterial {})),
                Transform::identity(),
            ),
            top_light.clone(),
        ],
        lights: vec![top_light],
        background: Box::new(ConstantColor {
            color: RgbColor::new(0.5, 0.5, 0.5),
        }),
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
pub fn temple() -> WorldInfo {
    let origin = Point3::new(-100.0f32, 100.0, -800.0);
    let look_at = Point3::new(64.0f32, 64.0, 64.0);
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
    let temple = OctTree::rectangle(Vector3::new(5, 100, 100), VoxelMaterial {}).combine(
        &OctTree::rectangle(Vector3::new(100, 5, 100), VoxelMaterial {}),
        Point3::new(0, 0, 0),
    );
    WorldInfo {
        objects: vec![
            Object::new(Box::new(temple), Transform::identity()),
            top_light.clone(),
        ],
        lights: vec![top_light],
        background: Box::new(ConstantColor {
            color: RgbColor::new(0.5, 0.5, 0.5),
        }),
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
pub fn temple_below() -> WorldInfo {
    let origin = Point3::new(-100.0f32, -100.0, -800.0);
    let look_at = Point3::new(64.0f32, 64.0, 64.0);
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
    let temple = OctTree::rectangle(Vector3::new(5, 100, 100), VoxelMaterial {}).combine(
        &OctTree::rectangle(Vector3::new(100, 5, 100), VoxelMaterial {}),
        Point3::new(0, 0, 0),
    );
    WorldInfo {
        objects: vec![
            Object::new(Box::new(temple), Transform::identity()),
            top_light.clone(),
        ],
        lights: vec![top_light],
        background: Box::new(ConstantColor {
            color: RgbColor::new(0.5, 0.5, 0.5),
        }),
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
pub fn cube() -> WorldInfo {
    let origin = Point3::new(-100.0f32, 100.0, -800.0);
    let look_at = Point3::new(64.0f32, 64.0, 64.0);
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
    let temple = OctTree::cube(4, VoxelMaterial {});
    WorldInfo {
        objects: vec![
            Object::new(Box::new(temple), Transform::identity()),
            top_light.clone(),
        ],
        lights: vec![top_light],
        background: Box::new(ConstantColor {
            color: RgbColor::new(0.1, 0.1, 0.1),
        }),
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
pub fn debug_cube() -> WorldInfo {
    let origin = Point3::new(-100.0f32, 100.0, -800.0);
    let look_at = Point3::new(64.0f32, 64.0, 64.0);
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
    let temple = OctTree::suboptimal_cube(VoxelMaterial {});
    WorldInfo {
        objects: vec![
            Object::new(Box::new(temple), Transform::identity()),
            top_light.clone(),
        ],
        lights: vec![top_light],
        background: Box::new(ConstantColor {
            color: RgbColor::new(0.1, 0.1, 0.1),
        }),
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
pub fn cube_back() -> WorldInfo {
    let origin = Point3::new(100.0f32, 100.0, 800.0);
    let look_at = Point3::new(64.0f32, 64.0, 64.0);
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
    let temple = OctTree::cube(4, VoxelMaterial {});
    WorldInfo {
        objects: vec![
            Object::new(Box::new(temple), Transform::identity()),
            top_light.clone(),
        ],
        lights: vec![top_light],
        background: Box::new(ConstantColor {
            color: RgbColor::new(0.1, 0.1, 0.1),
        }),
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

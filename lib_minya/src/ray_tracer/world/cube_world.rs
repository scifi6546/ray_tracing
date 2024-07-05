use super::{
    world_prelude::*, Camera, ConstantColor, DiffuseLight, Object, SolidColor, Sphere, Transform,
    VoxelWorld, WorldInfo,
};
use crate::prelude::*;
use cgmath::{prelude::*, Point3, Vector3};

pub fn cube_world() -> WorldInfo {
    let look_at = Point3::new(0.0f32, 5.0, 5.0);

    let origin = Point3::new(-20.0f32, 5.0, -20.0);

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
    //red_metal

    let mut world = VoxelWorld::new(
        vec![CubeMaterial::new(RgbColor::new(0.65, 0.05, 0.05))],
        vec![],
        10,
        10,
        10,
    );
    for i in 3..6 {
        for j in 3..6 {
            for k in 3..6 {
                world.update(i, j, k, CubeMaterialIndex::new_solid(0));
            }
        }
    }
    world.update(0, 0, 0, CubeMaterialIndex::new_solid(0));
    world.update(0, 1, 0, CubeMaterialIndex::new_solid(0));
    world.update(5, 5, 5, CubeMaterialIndex::new_solid(0));
    WorldInfo {
        objects: vec![
            Object::new(Box::new(world), Transform::identity()),
            light.clone(),
        ],
        lights: vec![light],
        background: Box::new(ConstantColor {
            color: 0.1 * RgbColor::new(1.0, 1.0, 1.0),
        }),
        camera: Camera::new(
            1.0,
            fov,
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

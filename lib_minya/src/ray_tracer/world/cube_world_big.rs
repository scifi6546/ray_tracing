use super::{
    world_prelude::*, Camera, DiffuseLight, Object, Sky, SolidColor, Sphere, Transform, VoxelWorld,
    WorldInfo,
};
use crate::prelude::*;
use cgmath::{prelude::*, Point2, Point3, Vector3};

pub fn cube_world_big() -> WorldInfo {
    let look_at = Point3::new(50.0f32, 10.0, 50.0);

    let origin = Point3::new(-20.0f32, 50.0, -20.0);

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
    let mut world = VoxelWorld::new(
        vec![
            CubeMaterial::new(RgbColor::new(0.65, 0.05, 0.05)),
            CubeMaterial::new(RgbColor::new(0.65, 0.8, 0.05)),
        ],
        vec![],
        100,
        MAX_Y,
        100,
    );

    for x in 0..100 {
        for z in 0..100 {
            let h = height(x, z);
            for y in 0..=h {
                let index = if y < 9 { 1 } else { 0 };
                world.update(x, y, z, CubeMaterialIndex::new_solid(index));
            }
        }
    }

    WorldInfo {
        objects: vec![
            Object::new(Box::new(world), Transform::identity()),
            lava_light.clone(),
        ],
        lights: vec![lava_light],
        background: Box::new(Sky { intensity: 0.1 }),
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

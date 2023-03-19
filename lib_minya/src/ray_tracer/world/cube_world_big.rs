use super::{
    Camera, ConstantColor, ConstantMedium, CubeWorld, DiffuseLight, Isotropic, Lambertian, Metal,
    Object, RenderBox, Sky, SolidColor, Sphere, Transform, WorldInfo, XYRect, XZRect, YZRect,
};
use crate::prelude::*;
use cgmath::{prelude::*, Point2, Point3, Vector2, Vector3};
use dyn_clone::clone_box;
use std::ops::Deref;
use std::{cell::RefCell, rc::Rc};

pub fn cube_world_big() -> WorldInfo {
    let look_at = Point3::new(50.0f32, 10.0, 50.0);

    let origin = Point3::new(-20.0f32, 50.0, -20.0);

    let fov = 40.0;
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    let red = Box::new(Lambertian {
        albedo: Box::new(SolidColor {
            color: RgbColor::new(0.65, 0.05, 0.05),
        }),
    });
    let light = Box::new(DiffuseLight {
        emit: Box::new(SolidColor {
            color: 20000.0 * RgbColor::WHITE,
        }),
    });

    let rect = Object::new(
        Box::new(XZRect::new(0.0, 5.0, 0.0, 5.0, 20.0, light.clone(), true)),
        Transform::identity(),
    );

    fn height(x: isize, z: isize) -> isize {
        let center = Point2::new(50.0, 50.0);
        let radius = center.distance(Point2::new(x as f32, z as f32));
        let h = (radius / 10.0).cos() * 10.0 + 15.0;
        h.max(0.0).min(11.0) as isize
    }
    let mut world = CubeWorld::new(red, 100, 12, 100);
    //world.update(0, 0, 0, true);
    for x in 0..100 {
        for z in 0..100 {
            let h = height(x, z);
            for y in 0..=h {
                world.update(x, y, z, true);
            }
        }
    }
    world.update(3, 6, 3, true);

    world.update(0, 0, 0, false);
    world.update(0, 1, 0, false);
    /*
    for x in 0..100 {
        for y in 0..5 {
            for z in 0..100 {
                world.update(x, y, z, true)
            }
        }
    }

     */
    WorldInfo {
        objects: vec![
            Object::new(Box::new(world), Transform::identity()),
            rect.clone(),
        ],
        lights: vec![rect],
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
    }
}

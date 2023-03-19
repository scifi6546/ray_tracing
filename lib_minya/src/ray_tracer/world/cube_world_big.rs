use super::{
    Camera, ConstantColor, ConstantMedium, CubeWorld, DiffuseLight, Isotropic, Lambertian, Metal,
    Object, RenderBox, Sky, SolidColor, Sphere, Transform, WorldInfo, XYRect, XZRect, YZRect,
};
use crate::prelude::*;
use cgmath::{prelude::*, Point3, Vector3};
use dyn_clone::clone_box;
use std::ops::Deref;
use std::{cell::RefCell, rc::Rc};

pub fn cube_world_big() -> WorldInfo {
    let look_at = Point3::new(0.0f32, 5.0, 5.0);

    let origin = Point3::new(-20.0f32, 20.0, -20.0);

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
        Box::new(XZRect::new(0.0, 5.0, 0.0, 5.0, 10.0, light.clone(), true)),
        Transform::identity(),
    );

    let white = Box::new(Lambertian {
        albedo: Box::new(SolidColor {
            color: RgbColor::new(0.73, 0.73, 0.73),
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
    let mut world = CubeWorld::new(red, 100, 10, 100);
    //world.update(0, 0, 0, true);
    for x in 0..6 {
        for y in 0..2 {
            for z in 0..6 {
                world.update(x, y, z, true);
            }
        }
    }
    world.update(3, 6, 3, true);
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

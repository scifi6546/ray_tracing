use super::{Camera, CameraInfo};
use crate::prelude::{RayScalar, RgbColor};
use crate::ray_tracer::background::Sky;

use crate::ray_tracer::hittable::{Object, OctTree, SolidVoxel, Transform, Voxel};
use crate::ray_tracer::world::WorldInfo;

use cgmath::{prelude::*, Point3, Vector3};

pub(crate) fn simple_cube() -> WorldInfo {
    let fov = 40.0;

    let look_at = Point3::new(0., 0., 0.);
    //let look_at = Point3::new(0.0, 0.0, 0.0);
    let origin = Point3::<RayScalar>::new(-20., -20., 20.);

    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    let mut tree = OctTree::<Voxel>::empty();
    for x in 0..3 {
        for y in 0..3 {
            for z in 0..3 {
                tree.set(
                    Point3 { x, y, z },
                    Voxel::Solid(SolidVoxel::Lambertian {
                        albedo: RgbColor::new(0.65, 0.05, 0.05),
                    }),
                )
            }
        }
    }

    WorldInfo {
        objects: vec![Object::new(Box::new(tree), Transform::identity())],
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
pub(crate) fn cube_recreation() -> WorldInfo {
    let fov = 40.0;

    let look_at = Point3::new(0., 0., 0.);
    //let look_at = Point3::new(0.0, 0.0, 0.0);
    let origin = Point3::<RayScalar>::new(-20., -20., 20.);

    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };
    let mat = Voxel::Solid(SolidVoxel::Lambertian {
        albedo: RgbColor::new(0.65, 0.05, 0.05),
    });
    let world: OctTree<Voxel> = OctTree::rectangle(Vector3::new(3, 3, 3), mat);

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

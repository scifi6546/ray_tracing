use super::{Camera, CameraInfo};
use crate::prelude::RayScalar;
use crate::ray_tracer::background::Sky;

use crate::ray_tracer::hittable::hittable_objects::{CubeMaterial, VoxelMap};
use crate::ray_tracer::hittable::{
    voxel_world::CubeMaterialIndex, Object, OctTree, Transform, VoxelMaterial, VoxelWorld,
};
use crate::ray_tracer::world::WorldInfo;
use base_lib::RgbColor;
use cgmath::{prelude::*, Point3, Vector3};
use log::info;
pub(crate) fn sinnoh() -> WorldInfo {
    let file = VoxelMap::load("./voxel_assets/sinnoh/twinleaf.yml");
    let tile_size_x = 16;
    let tile_size_z = 16;
    let num_tiles_x = file.num_tiles_x() as i32;
    let num_tiles_z = file.num_times_z() as i32;
    let block_x = tile_size_x * num_tiles_x;
    let block_y = 70;
    let block_z = tile_size_z * num_tiles_z;
    info!("block_x: {},block_Z: {}", block_x, block_z);
    let fov = 40.0;

    let look_at = Point3::new(block_x as RayScalar / 2.0, 10.0, block_z as RayScalar / 2.0);
    //let look_at = Point3::new(0.0, 0.0, 0.0);
    let origin = Point3::<RayScalar>::new(-500.0, 300.0, block_z as RayScalar / 2.0);

    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };
    let mut world = VoxelWorld::new(
        vec![CubeMaterial::new(RgbColor::WHITE)],
        vec![],
        block_x,
        block_y,
        block_z,
    );
    file.apply_to_world(&mut world);
    let world: OctTree<VoxelMaterial> = world.into();
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
/*
 1.0,
           fov,
           origin,
           look_at,
           Vector3::new(0.0, 1.0, 0.0),
           0.00001,
           focus_distance,
           0.0,
           0.0,
*/
pub(crate) fn simple_cube() -> WorldInfo {
    let fov = 40.0;

    let look_at = Point3::new(0., 0., 0.);
    //let look_at = Point3::new(0.0, 0.0, 0.0);
    let origin = Point3::<RayScalar>::new(-20., -20., 20.);

    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    let mut world = VoxelWorld::new(
        vec![CubeMaterial::new(RgbColor::new(0.65, 0.05, 0.05))],
        vec![],
        10,
        10,
        10,
    );
    for x in 0..3 {
        for y in 0..3 {
            for z in 0..3 {
                world.update(x, y, z, CubeMaterialIndex::Solid { index: 0 })
            }
        }
    }
    let world: OctTree<VoxelMaterial> = world.into();

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
pub(crate) fn cube_recreation() -> WorldInfo {
    let fov = 40.0;

    let look_at = Point3::new(0., 0., 0.);
    //let look_at = Point3::new(0.0, 0.0, 0.0);
    let origin = Point3::<RayScalar>::new(-20., -20., 20.);

    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    let world: OctTree<VoxelMaterial> = OctTree::rectangle(
        Vector3::new(3, 3, 3),
        VoxelMaterial {
            color: RgbColor::new(0.65, 0.05, 0.05),
        },
    );

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

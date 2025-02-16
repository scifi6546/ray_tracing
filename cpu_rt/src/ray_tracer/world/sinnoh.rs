use super::{hittable_objects::*, Camera, CameraInfo, Object, Sky, Transform, WorldInfo};
use crate::prelude::*;
use cgmath::prelude::*;
pub(crate) fn twinleaf_map() -> WorldInfo {
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

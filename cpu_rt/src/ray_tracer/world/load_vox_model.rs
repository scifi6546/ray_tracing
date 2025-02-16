use super::{
    hittable_objects::*, Camera, CameraInfo, Object, Sky, Transform, VoxelWorld, WorldInfo,
};
use crate::prelude::*;
use cgmath::{prelude::*, Point3, Vector3};

pub fn load_vox_model() -> WorldInfo {
    const BLOCK_X: i32 = 60;
    const BLOCK_Y: i32 = 50;
    const BLOCK_Z: i32 = 60;
    let model = VoxelModel::load("voxel_assets/apartment_building.vox");
    let look_at = Point3::new(BLOCK_X as RayScalar / 2.0, 10.0, BLOCK_Z as RayScalar / 2.0);

    let origin = Point3::<RayScalar>::new(-50.0, 100.0, -40.0);
    //let origin = Point3::new(-40.0f32, 10.0, -40.0);

    let fov = 40.0;
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    let materials = vec![CubeMaterial::new(RgbColor::WHITE)];
    let mut world = VoxelWorld::new(materials, vec![], BLOCK_X, BLOCK_Y, BLOCK_Z);
    model.add_to_world(&mut world, Point3::new(0, 0, 0));

    WorldInfo {
        objects: vec![Object::new(Box::new(world), Transform::identity())],
        lights: vec![],
        background: Box::new(Sky { intensity: 0.4 }),
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

use super::{
    hittable_objects::*, world_prelude::*, Camera, CubeWorld, DiffuseLight, Object, Sky,
    SolidColor, Sphere, Transform, WorldInfo,
};
use crate::prelude::*;
use cgmath::{num_traits::FloatConst, prelude::*, Point2, Point3, Vector3};
use rand::prelude::*;

pub fn load_vox_model() -> WorldInfo {
    const BLOCK_X: i32 = 60;
    const BLOCK_Y: i32 = 50;
    const BLOCK_Z: i32 = 60;
    let model = VoxelModel::load("voxel_assets/apartment_building.vox");
    let look_at = Point3::new(BLOCK_X as f32 / 2.0, 10.0, BLOCK_Z as f32 / 2.0);

    let origin = Point3::new(-50.0f32, 10.0, -40.0);

    let fov = 40.0;
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    let materials = vec![CubeMaterial::new(RgbColor::WHITE)];
    let mut world = CubeWorld::new(materials, vec![], BLOCK_X, BLOCK_Y, BLOCK_Z);
    model.add_to_world(&mut world);

    WorldInfo {
        objects: vec![Object::new(Box::new(world), Transform::identity())],
        lights: vec![],
        background: Box::new(Sky { intensity: 0.4 }),
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

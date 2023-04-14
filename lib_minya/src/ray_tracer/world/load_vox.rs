use super::{
    hittable_objects::*, world_prelude::*, Camera, CubeWorld, DiffuseLight, Object, Sky,
    SolidColor, Sphere, Transform, WorldInfo,
};
use crate::prelude::*;
use cgmath::{num_traits::FloatConst, prelude::*, Point2, Point3, Vector3};
use rand::prelude::*;
use std::collections::{HashMap, HashSet};

pub fn load_vox() -> WorldInfo {
    const BLOCK_X: i32 = 20;
    const BLOCK_Y: i32 = 50;
    const BLOCK_Z: i32 = 20;

    let look_at = Point3::new(BLOCK_X as f32 / 2.0, 10.0, BLOCK_Z as f32 / 2.0);

    let origin = Point3::new(50.0f32, 10.0, 40.0);

    let fov = 40.0;
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    let files = dot_vox::load("voxel_assets/building.vox").expect("voxel files");
    let mut used_indices: HashSet<u8> = HashSet::new();
    for m in files.models.iter() {
        for v in m.voxels.iter() {
            used_indices.insert(v.i);
        }
    }
    let mut materials: Vec<CubeMaterial> = Vec::new();
    let mut index_to_material: HashMap<u8, usize> = HashMap::new();
    for idx in used_indices.iter() {
        let color_u32 = files.palette[*idx as usize];
        let red = ((color_u32 & 0x00ff_00_00u32) >> 16) as f32 / 255.0;
        let green = ((color_u32 & 0x00_00_ff_00u32) >> 8) as f32 / 255.0;
        let blue = (color_u32 & 0x00_00_00_ffu32) as f32 / 255.0;
        let color = RgbColor::new(red, green, blue);

        let new_idx = materials.len();
        materials.push(CubeMaterial::new(color));
        index_to_material.insert(*idx, new_idx);
    }

    let mut world = CubeWorld::new(materials, vec![], BLOCK_X, BLOCK_Y, BLOCK_Z);

    let mut used_cube_mat_indices = HashSet::new();
    for m in files.models.iter() {
        for v in m.voxels.iter() {
            let index = index_to_material[&v.i] as u16;
            used_cube_mat_indices.insert(index);

            world.update(
                v.x as isize,
                v.z as isize,
                v.y as isize,
                CubeMaterialIndex::new_solid(index),
            )
        }
    }

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

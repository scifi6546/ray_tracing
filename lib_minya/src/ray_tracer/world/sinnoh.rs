use super::{hittable_objects::*, Camera, Object, Sky, Transform, WorldInfo};
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

    let look_at = Point3::new(block_x as f32 / 2.0, 10.0, block_z as f32 / 2.0);

    let origin = Point3::new(0.0f32, 30.0,  2.0);
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
pub(crate) fn twinleaf_town() -> WorldInfo {
    let tile_size_x = 16;
    let tile_size_z = 16;
    let map = [
        [
            0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0,
            0, 0, 0,
        ],
        [
            5, 0, 5, 0, 5, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 5, 0, 5,
            0, 5, 0,
        ],
        [
            0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0,
            0, 0, 0,
        ],
        [
            5, 0, 5, 0, 5, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 5, 0, 5,
            0, 5, 0,
        ],
        [
            0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0,
            0, 0, 0,
        ],
        [
            5, 0, 5, 0, 5, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 5, 0, 5,
            0, 5, 0,
        ],
        [
            0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0,
            0, 0, 0,
        ],
        [
            5, 0, 5, 0, 5, 0, 1, 1, 0, 0, 0, 0, 0, 1, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 5, 0, 5,
            0, 5, 0,
        ],
        [
            0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 2, 2, 2, 2, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0,
            0, 0, 0,
        ],
        [
            5, 0, 5, 0, 5, 0, 1, 1, 0, 0, 0, 0, 0, 1, 2, 2, 2, 2, 0, 0, 1, 1, 1, 1, 1, 1, 5, 0, 5,
            0, 5, 0,
        ],
        [
            0, 0, 0, 0, 0, 0, 1, 1, 4, 0, 0, 0, 0, 3, 2, 2, 2, 2, 6, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0,
            0, 0, 0,
        ],
        [
            5, 0, 5, 0, 5, 0, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 5, 0, 5,
            0, 5, 0,
        ],
        [
            0, 0, 0, 0, 0, 0, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 0, 0, 0,
            0, 0, 0,
        ],
        [
            5, 0, 5, 0, 5, 0, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 5, 0, 5,
            0, 5, 0,
        ],
        [
            0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0,
            0, 0, 0,
        ],
        [
            5, 0, 5, 0, 5, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 1, 1, 5, 0, 5,
            0, 5, 0,
        ],
        [
            0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 1, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0,
            0, 0, 0,
        ],
        [
            5, 0, 5, 0, 5, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 4, 0, 0, 0, 0, 1, 1, 5, 0, 5,
            0, 5, 0,
        ],
        [
            0, 0, 0, 0, 0, 0, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0,
            0, 0, 0,
        ],
        [
            5, 0, 5, 0, 5, 0, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 5, 0, 5,
            0, 5, 0,
        ],
        [
            0, 0, 0, 0, 0, 0, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 0, 0, 0,
            0, 0, 0,
        ],
        [
            5, 0, 5, 0, 5, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 5, 0, 5,
            0, 5, 0,
        ],
        [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        [
            5, 0, 5, 0, 5, 0, 5, 0, 5, 0, 5, 0, 3, 3, 3, 3, 3, 3, 3, 3, 5, 0, 5, 0, 5, 0, 5, 0, 5,
            0, 5, 0,
        ],
        [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        [
            5, 0, 5, 0, 5, 0, 5, 0, 5, 0, 5, 0, 3, 3, 3, 3, 3, 3, 3, 3, 5, 0, 5, 0, 5, 0, 5, 0, 5,
            0, 5, 0,
        ],
    ];
    let num_tiles_x = map.len() as i32;
    let num_tiles_z = map[0].len() as i32;
    let block_x = tile_size_x * num_tiles_x;
    let block_y = 70;
    let block_z = tile_size_z * num_tiles_z;
    info!("block_x: {},block_Z: {}", block_x, block_z);
    let fov = 40.0;

    let look_at = Point3::new(block_x as f32 / 2.0, 10.0, block_z as f32 / 2.0);
    let look_at = Point3::new(0.0, 0.0, 0.0);
    let origin = Point3::new(-500.0f32, 300.0, block_z as f32 / 2.0);

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

    let grass = VoxelModel::load("voxel_assets/sinnoh/grass.vox");
    let mail_box = VoxelModel::load("voxel_assets/sinnoh/mail_box.vox");
    let dirt = VoxelModel::load("voxel_assets/sinnoh/dirt.vox");
    let house = VoxelModel::load("voxel_assets/sinnoh/house.vox");
    let small_house = VoxelModel::load("voxel_assets/sinnoh/small_house.vox");
    let tree = VoxelModel::load("voxel_assets/sinnoh/small_tree.vox");
    for x in 0..num_tiles_x {
        for z in 0..num_tiles_z {
            let tile = match map[x as usize][z as usize] {
                0 => None,
                1 => Some(&grass),
                2 => Some(&dirt),
                3 => Some(&mail_box),
                4 => Some(&house),
                5 => Some(&tree),
                6 => Some(&small_house),
                _ => panic!(),
            };
            let x_tile_pos = num_tiles_x as isize - x as isize - 1;
            let z_tile_pos = z as isize;
            if let Some(tile) = tile {
                tile.add_to_world(
                    &mut world,
                    Point3::new(
                        x_tile_pos * tile_size_x as isize,
                        0,
                        z_tile_pos * tile_size_z as isize,
                    ),
                );
            }

            let material_colors = world.get_solid_material_colors();
        }
    }

    WorldInfo {
        objects: vec![Object::new(Box::new(world), Transform::identity())],
        lights: vec![],
        background: Box::new(Sky { intensity: 0.6 }),
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

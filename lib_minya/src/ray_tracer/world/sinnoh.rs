use super::{hittable_objects::*, Camera, Object, Sky, Transform, WorldInfo};
use crate::prelude::*;
use cgmath::prelude::*;

pub(crate) fn twinleaf_town() -> WorldInfo {
    let TILE_SIZE_X = 16;
    let TILE_SIZE_Z = 16;
    let map = [
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 2, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0],
        [0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0],
        [0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 0, 0, 0, 0, 0, 0, 0],
        [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0],
        [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0],
        [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    let NUM_TILES_X = map.len() as i32;
    let NUM_TILES_Z = map[0].len() as i32;
    let BLOCK_X = TILE_SIZE_X * NUM_TILES_X;
    let BLOCK_Y = 16;
    let BLOCK_Z = TILE_SIZE_Z * NUM_TILES_Z;
    info!("block_x: {},block_Z: {}", BLOCK_X, BLOCK_Z);
    let fov = 40.0;

    let look_at = Point3::new(BLOCK_X as f32 / 2.0, 10.0, BLOCK_Z as f32 / 2.0);

    let origin = Point3::new(-500.0f32, 300.0, BLOCK_Z as f32 / 2.0);
    let focus_distance = {
        let t = look_at - origin;
        (t.dot(t)).sqrt()
    };

    let mut world = CubeWorld::new(
        vec![CubeMaterial::new(RgbColor::WHITE)],
        vec![],
        BLOCK_X,
        BLOCK_Y,
        BLOCK_Z,
    );

    let grass = VoxelModel::load("voxel_assets/sinnoh/grass.vox");
    let mail_box = VoxelModel::load("voxel_assets/sinnoh/mail_box.vox");
    let dirt = VoxelModel::load("voxel_assets/sinnoh/dirt.vox");

    for x in 0..NUM_TILES_X {
        for z in 0..NUM_TILES_Z {
            let tile = match map[x as usize][z as usize] {
                0 => &grass,
                1 => &dirt,
                2 => &mail_box,
                _ => panic!(),
            };
            let x_tile_pos = NUM_TILES_X as isize - x as isize - 1;
            let z_tile_pos = z as isize;
            //info!("x_tile: {}, z_tile: {}", x_tile_pos, z_tile_pos);
            tile.add_to_world(
                &mut world,
                Point3::new(
                    x_tile_pos * TILE_SIZE_X as isize,
                    0,
                    z_tile_pos * TILE_SIZE_Z as isize,
                ),
            );

            let material_colors = world.get_solid_material_colors();
            //tall_grass 230
            //base 227
            /*
            for color in material_colors.iter() {
                info!("mat color: {}", color)
            }

             */
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

use super::{
    super::{material::SolidVoxel, Voxel},
    OctTree,
};
use crate::{ray_tracer::hittable::hittable_objects::VoxelMap, RgbColor};

use cgmath::Point3;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{Error as IoError, Read},
    ops::Neg,
    path::Path,
};
impl OctTree<Voxel> {
    pub fn load_map<P: AsRef<Path>>(p: P) -> Result<Self, IoError> {
        let file = File::open(p)?;
        let mut tree = OctTree::<Voxel>::empty();
        let map: VoxelMap = serde_yaml::from_reader(file).expect("failed to parse");
        let models = map
            .tile_types
            .iter()
            .map(|tile| {
                if let Some(model_path) = tile.model_path.as_ref() {
                    (
                        tile.index,
                        Some(Self::load_vox(model_path).expect("failed to find vox file")),
                    )
                } else {
                    (tile.index, None)
                }
            })
            .collect::<HashMap<_, _>>();
        for x in 0..map.tiles.len() {
            for z in 0..map.tiles[x].len() {
                let tile_index = map.tiles[x][z];
                let tile = models.get(&tile_index).expect("invalid tile type");
                if let Some(tile) = tile.as_ref() {
                    let x_tile_pos = map.num_tiles_x() - x - 1;
                    let old_tree = tree.clone();
                    tree = old_tree.combine(
                        tile,
                        Point3::new(
                            x_tile_pos as i32 * map.tile_size as i32,
                            0,
                            z as i32 * map.tile_size as i32,
                        ),
                    );
                }
            }
        }
        Ok(tree)
    }
    /// loads .vox files
    pub fn load_vox<P: AsRef<Path>>(load_path: P) -> Result<Self, IoError> {
        const MAX_FILE_SIZE: u64 = 1_000_000_000;
        let mut file = File::open(load_path)?;
        let mut bytes = Vec::<u8>::new();
        if file.metadata()?.len() > MAX_FILE_SIZE {
            panic!("voxel model is too big");
        }
        file.read_to_end(&mut bytes)?;
        let vox_data = dot_vox::load_bytes(&bytes).expect("failed to get");
        let mut used_indices = HashSet::<u8>::new();
        let mut min_x = usize::MAX;
        let mut max_x = usize::MIN;

        let mut min_y = usize::MAX;
        let mut max_y = usize::MIN;

        let mut min_z = usize::MAX;
        let mut max_z = usize::MIN;
        for model in vox_data.models.iter() {
            for v in model.voxels.iter() {
                used_indices.insert(v.i);
                min_x = (v.x as usize).min(min_x);
                max_x = (v.x as usize).max(max_x);

                min_y = (v.y as usize).min(min_y);
                max_y = (v.y as usize).max(max_y);

                min_z = (v.z as usize).min(min_z);
                max_z = (v.z as usize).max(max_z);
            }
        }
        let mut materials = Vec::<Voxel>::new();
        let mut index_to_material = HashMap::<u8, usize>::new();
        for idx in used_indices.iter() {
            let color = vox_data.palette[*idx as usize];

            let red = color.r as f32 / 255.0;
            let green = color.g as f32 / 255.0;
            let blue = color.b as f32 / 255.0;

            let color = RgbColor::new(red, green, blue);

            let new_idx = materials.len();

            materials.push(Voxel::Solid(SolidVoxel::Lambertian { albedo: color }));
            index_to_material.insert(*idx, new_idx);
        }

        let mut tree = OctTree::<Voxel>::empty();
        for model in vox_data.models {
            for voxel in model.voxels.iter() {
                let material_index = index_to_material[&voxel.i];
                let material = materials[material_index];
                let position = Point3 {
                    x: voxel.x as i32 - min_x as i32,
                    y: voxel.z as i32 - min_z as i32,
                    z: (voxel.y as i32 - min_y as i32).neg() + max_y as i32 - min_y as i32,
                }
                .map(|v| v as u32);
                tree.set(position, material);
            }
        }
        Ok(tree)
    }
}

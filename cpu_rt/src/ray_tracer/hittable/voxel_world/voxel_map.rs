use super::{VoxelModel, VoxelWorld};
use cgmath::Point3;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File};
#[derive(Deserialize, Serialize)]
pub(crate) struct TileType {
    pub(crate) index: u32,
    pub(crate) is_air: bool,
    pub(crate) model_path: Option<String>,
}
impl TileType {
    pub(crate) fn is_valid(&self) -> bool {
        if self.is_air {
            self.model_path.is_none()
        } else {
            if let Some(model_path) = self.model_path.as_ref() {
                let path = std::path::Path::new(model_path);
                path.exists()
            } else {
                false
            }
        }
    }
}
#[derive(Deserialize, Serialize)]
pub(crate) struct VoxelMap {
    pub(crate) tile_types: Vec<TileType>,
    pub(crate) tile_size: u32,
    /// outer array is x, inner is z, so indexing looks like `tiles[x][z]`
    pub(crate) tiles: Vec<Vec<u32>>,
}
impl VoxelMap {
    pub(crate) fn load<P: AsRef<std::path::Path>>(p: P) -> Self {
        let file = File::open(p).expect("failed to open file");
        let s: VoxelMap = serde_yaml::from_reader(file).expect("failed to parse");

        assert!(s
            .tile_types
            .iter()
            .map(|t| t.is_valid())
            .fold(true, |acc, x| acc && x));
        s
    }
    pub(crate) fn num_tiles_x(&self) -> usize {
        self.tiles.len()
    }
    pub(crate) fn num_times_z(&self) -> usize {
        self.tiles.iter().map(|row| row.len()).max().unwrap()
    }
    pub(crate) fn apply_to_world(&self, voxels: &mut VoxelWorld) {
        let models = self
            .tile_types
            .iter()
            .map(|tile| {
                if let Some(model_path) = tile.model_path.as_ref() {
                    (tile.index, Some(VoxelModel::load(model_path)))
                } else {
                    (tile.index, None)
                }
            })
            .collect::<HashMap<_, _>>();
        for x in 0..self.tiles.len() {
            for z in 0..self.tiles[x].len() {
                let tile_index = self.tiles[x][z];
                let tile = models.get(&tile_index).expect("invalid tile type");
                if let Some(tile) = tile.as_ref() {
                    let x_tile_pos = self.num_tiles_x() - x - 1;
                    tile.add_to_world(
                        voxels,
                        Point3::new(
                            x_tile_pos as isize * self.tile_size as isize,
                            0,
                            z as isize * self.tile_size as isize,
                        ),
                    )
                }
            }
        }
    }
}

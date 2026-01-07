use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub(crate) struct TileType {
    pub(crate) index: u32,
    pub(crate) is_air: bool,
    pub(crate) model_path: Option<String>,
}
impl TileType {}
#[derive(Deserialize, Serialize)]
pub(crate) struct VoxelMap {
    pub(crate) tile_types: Vec<TileType>,
    pub(crate) tile_size: u32,
    /// outer array is x, inner is z, so indexing looks like `tiles[x][z]`
    pub(crate) tiles: Vec<Vec<u32>>,
}
impl VoxelMap {
    pub(crate) fn num_tiles_x(&self) -> usize {
        self.tiles.len()
    }
}

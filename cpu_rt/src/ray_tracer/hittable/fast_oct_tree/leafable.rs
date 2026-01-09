use super::voxel::{Voxel, VoxelMaterial};
use std::clone::Clone;
pub trait Leafable: Clone {
    type Material;
}
impl Leafable for Voxel {
    type Material = VoxelMaterial;
}
impl Leafable for u32 {
    type Material = ();
}

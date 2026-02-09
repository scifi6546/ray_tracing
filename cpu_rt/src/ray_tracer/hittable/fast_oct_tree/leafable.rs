use super::voxel::{Voxel, VoxelMaterial};
use cgmath::Point3;
use std::clone::Clone;
pub trait Leafable: Clone + PartialEq + std::fmt::Debug {
    type Material;
}
impl Leafable for Voxel {
    type Material = VoxelMaterial;
}
impl Leafable for u32 {
    type Material = ();
}
impl Leafable for Point3<u32> {
    type Material = ();
}

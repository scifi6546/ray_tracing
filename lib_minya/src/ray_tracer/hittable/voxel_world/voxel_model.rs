use super::{CubeMaterial, CubeMaterialIndex, Voxels};
use cgmath::Point3;
pub struct VoxelModel {
    model: Voxels<CubeMaterialIndex>,
    offset: Point3<i32>,
    solid_materials: Vec<CubeMaterial>,
    translucent_materials: Vec<CubeMaterial>,
}

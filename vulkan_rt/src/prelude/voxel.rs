use super::mesh::Model;
#[derive(Copy, Clone, Debug)]
pub enum Voxel {
    Air,
    Solid,
}
pub struct VoxelChunk {
    voxels: [Voxel; Self::SIZE * Self::SIZE * Self::SIZE],
}
impl VoxelChunk {
    pub const SIZE: usize = 16;
    pub fn new(ctor: impl Fn(i32, i32, i32) -> Voxel) -> Self {
        let mut voxels = [Voxel::Air; Self::SIZE * Self::SIZE * Self::SIZE];
        for x in 0..Self::SIZE {
            for y in 0..Self::SIZE {
                for z in 0..Self::SIZE {
                    voxels[x * Self::SIZE * Self::SIZE + y * Self::SIZE + z] =
                        ctor(x as i32, y as i32, z as i32);
                }
            }
        }
        Self { voxels }
    }
    pub fn build_mesh(&self) -> Model {
        let triangles: Vec<[f32; 3]> = Vec::new();
        todo!()
    }
}

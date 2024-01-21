use crate::prelude::{AnimationList, Mesh, StaticPosition, Vector2, Vector4, Vertex};
use base_lib::VoxelGrid;
use cgmath::{Point3, Vector3};
use std::{collections::BTreeMap, rc::Rc};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Voxel {
    Air,
    Solid,
}
pub struct VoxelChunk {
    voxels: Box<[Voxel; Self::SIZE_X * Self::SIZE_Y * Self::SIZE_Z]>,
}
impl VoxelChunk {
    pub const SIZE_X: usize = 64;
    pub const SIZE_Y: usize = Self::SIZE_X;
    pub const SIZE_Z: usize = Self::SIZE_X;
    pub fn new(ctor: impl Fn(i32, i32, i32) -> Voxel) -> Self {
        let mut voxels = Box::new([Voxel::Air; Self::SIZE_X * Self::SIZE_Y * Self::SIZE_Z]);
        for x in 0..Self::SIZE_X {
            for y in 0..Self::SIZE_Y {
                for z in 0..Self::SIZE_Z {
                    voxels[Self::get_idx(x, y, z)] = ctor(x as i32, y as i32, z as i32);
                }
            }
        }
        Self { voxels }
    }
    fn get_idx(x: usize, y: usize, z: usize) -> usize {
        x * Self::SIZE_Y * Self::SIZE_Z + y * Self::SIZE_Y + z
    }
    pub fn get_unchecked(&self, x: usize, y: usize, z: usize) -> Voxel {
        self.voxels[Self::get_idx(x, y, z)]
    }
    pub fn build_mesh(&self, offset: Vector3<f32>) -> Option<Mesh> {
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let offset: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
        for x in 0..Self::SIZE_X {
            for y in 0..Self::SIZE_Y {
                for z in 0..Self::SIZE_Z {
                    if self.get_unchecked(x, y, z) == Voxel::Solid {
                        let mut temp_vertices = [
                            Vertex {
                                pos: Vector4::new(
                                    0.0f32 + x as f32 + offset.x,
                                    0.0 + y as f32 + offset.y,
                                    1.0 + z as f32 + offset.z,
                                    1.0,
                                ),
                                uv: Vector2::new(0.0, 0.0),
                            },
                            Vertex {
                                pos: Vector4::new(
                                    1.0 + x as f32 + offset.x,
                                    0.0 + y as f32 + offset.y,
                                    1.0 + z as f32 + offset.z,
                                    1.0,
                                ),
                                uv: Vector2::new(0.0, 0.0),
                            },
                            Vertex {
                                pos: Vector4::new(
                                    0.0 + x as f32 + offset.x,
                                    1.0 + y as f32 + offset.y,
                                    1.0 + z as f32 + offset.z,
                                    1.0,
                                ),
                                uv: Vector2::new(0.0, 0.0),
                            },
                            Vertex {
                                pos: Vector4::new(
                                    1.0 + x as f32 + offset.x,
                                    1.0 + y as f32 + offset.y,
                                    1.0 + z as f32 + offset.z,
                                    1.0,
                                ),
                                uv: Vector2::new(0.0, 0.0),
                            },
                            Vertex {
                                pos: Vector4::new(
                                    0.0 + x as f32 + offset.x,
                                    0.0 + y as f32 + offset.y,
                                    0.0 + z as f32 + offset.z,
                                    1.0,
                                ),
                                uv: Vector2::new(0.0, 0.0),
                            },
                            Vertex {
                                pos: Vector4::new(
                                    1.0 + x as f32 + offset.x,
                                    0.0 + y as f32 + offset.y,
                                    0.0 + z as f32 + offset.z,
                                    1.0,
                                ),
                                uv: Vector2::new(0.0, 0.0),
                            },
                            Vertex {
                                pos: Vector4::new(
                                    0.0 + x as f32 + offset.x,
                                    1.0 + y as f32 + offset.y,
                                    0.0 + z as f32 + offset.z,
                                    1.0,
                                ),
                                uv: Vector2::new(0.0, 0.0),
                            },
                            Vertex {
                                pos: Vector4::new(
                                    1.0 + x as f32 + offset.x,
                                    1.0 + y as f32 + offset.y,
                                    1.0 + z as f32 + offset.z,
                                    1.0,
                                ),
                                uv: Vector2::new(0.0, 0.0),
                            },
                        ];
                        let mut temp_indices = vec![
                            0u32, 1, 2, 1, 3, 2, 2, 3, 7, 2, 7, 6, 1, 7, 3, 1, 5, 7, 6, 7, 4, 7, 5,
                            4, 0, 4, 1, 1, 4, 5, 2, 6, 4, 0, 2, 4,
                        ]
                        .iter()
                        .map(|i| i + vertices.len() as u32)
                        .collect();
                        for v in temp_vertices {
                            vertices.push(v)
                        }

                        indices.append(&mut temp_indices);
                    }
                }
            }
        }
        if indices.is_empty() {
            None
        } else {
            Some(Mesh::new(vertices, indices))
        }
    }
}
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct PositionKey {
    x: usize,
    y: usize,
    z: usize,
}
impl PositionKey {
    fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }
}
pub struct VoxelWorld {
    /// grid if chunks where each key is the location of the VoxelChunks zero point
    chunks: BTreeMap<PositionKey, VoxelChunk>,
}
impl VoxelWorld {
    pub fn from_voxel_grid(grid: &VoxelGrid) -> Self {
        let max_x = grid.size_x().div_ceil(VoxelChunk::SIZE_X);
        let max_y = grid.size_y().div_ceil(VoxelChunk::SIZE_Y);
        let max_z = grid.size_z().div_ceil(VoxelChunk::SIZE_Z);
        let chunks = (0..max_x)
            .flat_map(|key_x| {
                (0..max_y).flat_map(move |key_y| {
                    (0..max_z).map(move |key_z| {
                        (
                            PositionKey::new(
                                key_x * VoxelChunk::SIZE_X,
                                key_y * VoxelChunk::SIZE_Y,
                                key_z * VoxelChunk::SIZE_Z,
                            ),
                            VoxelChunk::new(|x, y, z| {
                                let grid_x = key_x * VoxelChunk::SIZE_X + x as usize;
                                let grid_y = key_y * VoxelChunk::SIZE_Y + y as usize;
                                let grid_z = key_z * VoxelChunk::SIZE_Z + z as usize;
                                if grid.get_tile(Point3::new(grid_x, grid_y, grid_z)) {
                                    Voxel::Solid
                                } else {
                                    Voxel::Air
                                }
                            }),
                        )
                    })
                })
            })
            .collect();
        Self { chunks }
    }
    pub fn build_model(&self) -> Vec<(Mesh, AnimationList)> {
        self.chunks
            .iter()
            .map(|(root_pos, chunk)| {
                (
                    chunk.build_mesh(Vector3::new(
                        root_pos.x as f32,
                        root_pos.y as f32,
                        root_pos.z as f32,
                    )),
                    AnimationList::new(vec![Rc::new(StaticPosition {
                        position: Point3::new(
                            root_pos.x as f32,
                            root_pos.y as f32,
                            root_pos.z as f32,
                        ),
                    })]),
                )
            })
            .filter(|(mesh, list)| mesh.is_some())
            .map(|(mesh, list)| (mesh.unwrap(), list))
            .collect()
    }
}

use crate::prelude::*;

use cgmath::{Point3, Vector3};
pub(crate) use voxel_map::VoxelMap;

mod voxel_map;

#[derive(Clone)]
struct Voxels<T: Clone> {
    data: Vec<T>,
    x_dim: usize,
    y_dim: usize,
    z_dim: usize,
}

impl<T: Clone + std::fmt::Debug> Voxels<T> {
    /// gets size of voxel grid
    pub(crate) fn size(&self) -> Vector3<usize> {
        Vector3::new(self.x_dim, self.y_dim, self.z_dim)
    }
    pub fn new(x_dim: usize, y_dim: usize, z_dim: usize, default_value: T) -> Self {
        Self {
            data: vec![default_value; x_dim * y_dim * z_dim],
            x_dim,
            y_dim,
            z_dim,
        }
    }
    fn get_idx(&self, x: usize, y: usize, z: usize) -> usize {
        x + y * self.x_dim + z * self.x_dim * self.y_dim
    }
    pub fn in_range(&self, x: isize, y: isize, z: isize) -> bool {
        x >= 0
            && y >= 0
            && z >= 0
            && x < self.x_dim as isize
            && y < self.y_dim as isize
            && z < self.z_dim as isize
    }
    pub fn get(&self, x: usize, y: usize, z: usize) -> T {
        self.data[self.get_idx(x, y, z)].clone()
    }
    pub fn update(&mut self, x: isize, y: isize, z: isize, val: T) {
        if self.in_range(x, y, z) {
            let idx = self.get_idx(x as usize, y as usize, z as usize);
            self.data[idx] = val;
        } else {
            error!("out of range ({}, {}, {})", x, y, z)
        }
    }
}

type MaterialIndex = u16;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum CubeMaterialIndex {
    Solid { index: MaterialIndex },
}
impl CubeMaterialIndex {
    pub fn new_air() -> Self {
        Self::Solid {
            index: MaterialIndex::MAX,
        }
    }
    pub fn is_solid(&self) -> bool {
        match self {
            Self::Solid { index } => *index != MaterialIndex::MAX,
        }
    }
    pub fn is_air(&self) -> bool {
        !self.is_solid()
    }
}

#[derive(Clone)]
pub(crate) struct CubeMaterial {
    color: RgbColor,
}
impl CubeMaterial {
    pub fn color(&self) -> RgbColor {
        self.color
    }
}
impl std::fmt::Debug for CubeMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cube Material")
            .field("color", &self.color)
            .finish()
    }
}

impl CubeMaterial {
    pub fn new(color: RgbColor) -> Self {
        CubeMaterial { color }
    }
}
#[derive(Clone)]
pub(crate) struct VoxelWorld {
    solid_materials: Vec<CubeMaterial>,
    voxels: Voxels<CubeMaterialIndex>,
}
impl VoxelWorld {
    /// gets witdh height and depth of voxel world
    pub(crate) fn size(&self) -> Vector3<u32> {
        self.voxels.size().map(|val| val as u32)
    }
    /// tries to get the material if it is in bounds of the world
    pub fn get(&self, position: Point3<u32>) -> Option<CubeMaterialIndex> {
        let size = self.size();
        if position.x < size.x && position.y < size.y && position.z < size.z {
            Some(self.voxels.get(
                position.x as usize,
                position.y as usize,
                position.z as usize,
            ))
        } else {
            None
        }
    }
    pub(crate) fn get_solid_material(&self, index: MaterialIndex) -> Option<CubeMaterial> {
        if (index as usize) < self.solid_materials.len() {
            Some(self.solid_materials[index as usize].clone())
        } else {
            None
        }
    }
    pub(crate) fn new(solid_materials: Vec<CubeMaterial>, x: i32, y: i32, z: i32) -> Self {
        Self {
            solid_materials,
            voxels: Voxels::new(
                x as usize,
                y as usize,
                z as usize,
                CubeMaterialIndex::new_air(),
            ),
        }
    }
    pub fn update(&mut self, x: isize, y: isize, z: isize, val: CubeMaterialIndex) {
        match val {
            CubeMaterialIndex::Solid { index } => {
                if index == MaterialIndex::MAX || (index as usize) < self.solid_materials.len() {
                    self.voxels.update(x, y, z, val)
                } else {
                    error!("invalid cube material index: {}", index)
                }
            }
        };
    }
}

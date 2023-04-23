use super::{CubeMaterial, CubeMaterialIndex, CubeWorld, RgbColor, Voxels};
use crate::prelude::*;
use crate::ray_tracer::hittable::voxel_world::Solid;
use cgmath::Point3;
use std::io::Read;
use std::{
    collections::{HashMap, HashSet},
    fs::*,
    path::Path,
};
#[derive(Clone)]
pub struct VoxelModel {
    model: Voxels<CubeMaterialIndex>,

    solid_materials: Vec<CubeMaterial>,
}
impl VoxelModel {
    const MAX_COLOR_DISTANCE: f32 = 0.01;

    pub fn load<P: AsRef<Path>>(p: P) -> Self {
        let mut f = File::open(p).expect("failed to get file");
        let mut bytes = vec![];
        f.read_to_end(&mut bytes).expect("failed to read");
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
        let mut materials: Vec<CubeMaterial> = Vec::new();
        let mut index_to_material = HashMap::<u8, usize>::new();
        for idx in used_indices.iter() {
            let color_u32 = vox_data.palette[*idx as usize];

            let blue = ((color_u32 & 0x00ff_00_00u32) >> 16) as f32 / 255.0;
            let green = ((color_u32 & 0x00_00_ff_00u32) >> 8) as f32 / 255.0;
            let red = (color_u32 & 0x00_00_00_ffu32) as f32 / 255.0;
            let color = RgbColor::new(red, green, blue);

            let new_idx = materials.len();
            materials.push(CubeMaterial::new(color));
            index_to_material.insert(*idx, new_idx);
        }

        let x_dim = (max_x - min_x) + 1;
        let y_dim = (max_z - min_z) + 1;
        let z_dim = (max_y - min_y) + 1;
        let mut world = Voxels::new(x_dim, y_dim, z_dim, CubeMaterialIndex::new_air());
        for model in vox_data.models.iter() {
            for voxel in model.voxels.iter() {
                let index = index_to_material[&voxel.i] as u16;

                world.update(
                    voxel.x as isize - min_x as isize,
                    voxel.z as isize - min_z as isize,
                    voxel.y as isize - min_y as isize,
                    CubeMaterialIndex::new_solid(index),
                )
            }
        }
        Self {
            model: world,
            solid_materials: materials,
        }
    }
    pub(crate) fn debug_text(&self) -> String {
        let mut string = String::new();
        for x in 0..self.model.x_dim {
            for y in 0..self.model.y_dim {
                for z in 0..self.model.z_dim {
                    let mat_idx = self.model.get(x, y, z);
                    let material = match mat_idx {
                        CubeMaterialIndex::Solid { index } => {
                            if mat_idx.is_solid() {
                                Some(self.solid_materials[index as usize].clone())
                            } else {
                                None
                            }
                        }
                        CubeMaterialIndex::Translucent { .. } => None,
                    };
                    if let Some(material) = material {
                        let txt = format!("x: {}, y: {}, z: {},mat: {}\n", x, y, z, material.color);
                        string += &txt;
                    }
                }
            }
        }
        string
    }
    pub fn add_to_world(&self, voxel_world: &mut CubeWorld, offset: Point3<isize>) {
        // old materials to new materials, key is index of old material, value is new index
        let mut material_indices = HashMap::<usize, usize>::new();
        let mut add_materials: Vec<CubeMaterial> = Vec::new();
        for (old_mat_index, old_material) in self.solid_materials.iter().enumerate() {
            let mut found_color = false;
            for (world_mat_index, world_mat) in voxel_world.solid_materials.iter().enumerate() {
                let dist = world_mat.distance(old_material);
                //info!("dist: {}", dist);
                if world_mat.distance(old_material) <= Self::MAX_COLOR_DISTANCE {
                    material_indices.insert(old_mat_index, world_mat_index);
                    found_color = true;
                    break;
                }
            }
            if !found_color {
                add_materials.push(old_material.clone());

                let index = voxel_world.solid_materials.len() + add_materials.len() - 1;

                material_indices.insert(old_mat_index, index);
            }
        }
        voxel_world.solid_materials.append(&mut add_materials);
        for x in 0..self.model.x_dim {
            for y in 0..self.model.y_dim {
                for z in 0..self.model.z_dim {
                    let voxel_mat = self.model.get(x, y, z);
                    if voxel_mat.is_solid()
                        && voxel_world.in_world(x as isize, y as isize, z as isize)
                    {
                        let mat_index = match voxel_mat {
                            CubeMaterialIndex::Solid { index } => {
                                material_indices[&(index as usize)] as u16
                            }
                            CubeMaterialIndex::Translucent { .. } => panic!(),
                        };
                        let material = CubeMaterialIndex::new_solid(mat_index);
                        voxel_world.update(
                            x as isize + offset.x,
                            y as isize + offset.y,
                            z as isize + offset.z,
                            material,
                        );
                    }
                }
            }
        }
    }
}

use super::{OctTree, VoxelMaterial};
use crate::ray_tracer::hittable::{voxel_world::CubeMaterialIndex, VoxelWorld};

use cgmath::Point3;
use log::{error, info};

impl From<VoxelWorld> for OctTree<VoxelMaterial> {
    fn from(old_world: VoxelWorld) -> Self {
        let mut world = OctTree::<VoxelMaterial>::empty();
        let size = old_world.size();
        info!("old world size: ({},{},{})", size.x, size.y, size.z);

        for x in 0..size.x {
            for y in 0..size.y {
                for z in 0..size.z {
                    let get_point = Point3::new(x, y, z);

                    let material = old_world.get(get_point);
                    if material.is_none() {
                        error!("failed to get cube at {}, {}, {}", x, y, z);
                        continue;
                    }
                    let material = material.unwrap();
                    if material.is_air() {
                        continue;
                    }
                    let update_material = match material {
                        CubeMaterialIndex::Solid { index } => {
                            let old_material = old_world.get_solid_material(index).unwrap();

                            VoxelMaterial {
                                color: old_material.color(),
                            }
                        }
                        CubeMaterialIndex::Translucent { .. } => todo!("translucent"),
                    };
                    world.update(get_point, update_material);
                }
            }
        }
        if !world.is_optimal(true) {
            error!("world is not optimally packed");
        }
        world
    }
}

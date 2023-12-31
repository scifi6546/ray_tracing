use super::{RuntimeMesh, VoxelGrid};
use crate::mesh::Mesh;
use cgmath::{Point3, Vector3};

pub(crate) struct Satellite {
    voxel: VoxelGrid<bool>,
    mesh: RuntimeMesh,
}
impl Satellite {
    pub fn new(device: &wgpu::Device) -> Self {
        let mut voxel = VoxelGrid::new(Vector3::new(8, 8, 8), false);
        for x in 0..8 {
            for y in 0..8 {
                for z in 0..8 {
                    voxel.set(Point3::new(x, y, z), true);
                }
            }
        }

        let mesh = RuntimeMesh::from_mesh(&voxel.build_mesh(), device);
        Self { voxel, mesh }
    }
    pub(crate) fn mesh(&self) -> &RuntimeMesh {
        &self.mesh
    }
}
#[derive(Clone, Debug)]
struct SatelliteData {}
pub(crate) struct SatelliteGrid {
    satellites: VoxelGrid<Option<SatelliteData>>,
    meshes: Vec<Mesh>,
}
impl SatelliteGrid {
    pub(crate) fn new() -> Self {
        Self {
            satellites: VoxelGrid::new(Vector3::new(10, 10, 10), None),
            meshes: vec![],
        }
    }
    pub(crate) fn get_meshes(&self) -> &[Mesh] {
        &self.meshes
    }
}

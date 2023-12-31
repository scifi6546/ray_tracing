use super::mesh::Mesh;
use cgmath::{Point3, Vector3};
pub(crate) struct VoxelGrid<T: std::clone::Clone> {
    data: Vec<T>,
    size: Vector3<usize>,
}
impl<T: std::clone::Clone> VoxelGrid<T> {
    pub(crate) fn new(size: Vector3<usize>, default_value: T) -> Self {
        let data = vec![default_value; size.x * size.y * size.z];
        Self { data, size }
    }
    fn get_idx(&self, point: Point3<usize>) -> usize {
        point.x + point.y * self.size.x + point.z * self.size.x * self.size.y
    }
    pub(crate) fn get(&self, point: Point3<usize>) -> T {
        self.data[self.get_idx(point)].clone()
    }
    pub(crate) fn set(&mut self, point: Point3<usize>, val: T) {
        let idx = self.get_idx(point);
        self.data[idx] = val;
    }
}
impl VoxelGrid<bool> {
    pub(crate) fn build_mesh(&self) -> Mesh {
        let mut out_mesh = Mesh::empty();
        for x in 0..self.size.x {
            for y in 0..self.size.y {
                for z in 0..self.size.z {
                    if self.get(Point3::new(x, y, z)) {
                        let mut cube = Mesh::cube();
                        cube.add_offset(Point3::new(x as f32, y as f32, z as f32));
                        out_mesh.add_mesh(&cube);
                    }
                }
            }
        }
        out_mesh
    }
}

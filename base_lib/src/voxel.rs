use cgmath::Vector3;
#[derive(Clone, Debug, PartialEq)]
pub struct VoxelGrid {
    grid: Vec<bool>,
    size_x: usize,
    size_y: usize,
    size_z: usize,
    center: Vector3<f32>,
}
impl VoxelGrid {
    pub fn new<F: Fn(usize, usize, usize) -> bool>(
        size: Vector3<usize>,
        center: Vector3<f32>,
        ctor_fn: F,
    ) -> Self {
        eprintln!("todo build voxels");
        Self {
            size_x: size.x,
            size_y: size.y,
            size_z: size.z,
            center,
            grid: Vec::new(),
        }
    }
}

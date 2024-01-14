use cgmath::{Point3, Vector3};
#[derive(Clone, Debug, PartialEq)]
pub struct VoxelGrid {
    grid: Vec<bool>,
    size_x: usize,
    size_y: usize,
    size_z: usize,
    center: Point3<f32>,
}
impl VoxelGrid {
    pub fn new<F: Fn(Point3<usize>) -> bool>(
        size: Vector3<usize>,
        center: Point3<f32>,
        ctor_fn: F,
    ) -> Self {
        eprintln!("todo build voxels");
        let length = size.x * size.y * size.z;
        Self {
            size_x: size.x,
            size_y: size.y,
            size_z: size.z,
            center,
            grid: (0..length)
                .map(|i| Self::get_pos_from_idx(i, size))
                .map(|pos| ctor_fn(pos))
                .collect(),
        }
    }
    pub fn size_x(&self) -> usize {
        self.size_x
    }
    pub fn size_y(&self) -> usize {
        self.size_y
    }
    pub fn size_z(&self) -> usize {
        self.size_z
    }
    pub fn get_tile(&self, pos: Point3<usize>) -> bool {
        if let Some(idx) = Self::get_idx(pos, Vector3::new(self.size_x, self.size_y, self.size_z)) {
            self.grid[idx]
        } else {
            false
        }
    }
    fn get_idx(pos: Point3<usize>, size: Vector3<usize>) -> Option<usize> {
        if pos.x < size.x && pos.y < size.y && pos.z < size.z {
            Some(pos.x * size.y * size.z + pos.y * size.z + pos.z)
        } else {
            None
        }
    }
    fn get_pos_from_idx(idx: usize, size: Vector3<usize>) -> Point3<usize> {
        Point3::new(
            idx / (size.y * size.z),
            (idx % (size.y * size.z)) / size.x,
            idx % size.z,
        )
    }
}

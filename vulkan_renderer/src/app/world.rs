use super::super::utils::{Matrix4, Vector3, Vector4};
pub struct World {
    frame_index: u64,
    pub camera: Camera,
}
impl World {
    pub fn update(&mut self) {
        self.frame_index += 1;
        let x = (self.frame_index as f32 / 1000.).sin() * 5.;
        let y = (self.frame_index as f32 / 1230.).sin() * 5.;
        self.camera.origin.x = x;
        self.camera.origin.y = y;
    }
}
impl Default for World {
    fn default() -> Self {
        Self {
            frame_index: 0,
            camera: Camera::new(),
        }
    }
}
pub struct Camera {
    pub origin: Vector3,
}
impl Camera {
    pub fn new() -> Self {
        Self {
            origin: Vector3::new(6., 1., -10.),
        }
    }
    pub fn to_matrix(&self) -> Matrix4 {
        Matrix4::translation(self.origin)
    }
}

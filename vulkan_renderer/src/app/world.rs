use super::super::utils::Vector4;
pub struct World {
    frame_index: u64,
    pub camera: Camera,
}
impl World {
    pub fn update(&mut self) {
        self.frame_index += 1;
        let x = (self.frame_index as f32 / 1000.).sin() * 5.;
        let y = (self.frame_index as f32 / 1230.).sin() * 5.;
        self.camera.origin.data[0] = x;
        self.camera.origin.data[1] = y;
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
    pub origin: Vector4,
}
impl Camera {
    pub fn new() -> Self {
        Self {
            origin: Vector4::new(6., 1., -10., 1.),
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        self.origin
            .data
            .iter()
            .flat_map(|v| v.to_ne_bytes())
            .collect()
    }
}

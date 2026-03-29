use super::super::utils::{Matrix4, Rotation, Vector3};
pub enum WorldEvent {
    Roll { amount: f32 },
    Yaw { amount: f32 },
    Pitch { amount: f32 },
}
pub struct World {
    frame_index: u64,
    pub camera: Camera,
}
impl World {
    /// world tick, updates every frame
    pub fn update(&mut self) {
        self.frame_index += 1;
        //let x = (self.frame_index as f32 / 1000.).sin() * 5.;
        //let y = (self.frame_index as f32 / 1230.).sin() * 5.;
        //self.camera.origin.x = x;
        //self.camera.origin.y = y;
    }
    pub fn react_event(&mut self, event: WorldEvent) {
        match event {
            WorldEvent::Roll { amount } => self.camera.rotation.roll += 0.1 * amount,
            WorldEvent::Yaw { amount } => self.camera.rotation.yaw += 0.05 * amount,
            WorldEvent::Pitch { amount } => self.camera.rotation.pitch += 0.5 * amount,
        }
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
    pub rotation: Rotation,
}
impl Camera {
    pub fn new() -> Self {
        Self {
            origin: Vector3::new(6., 1., -10.),
            rotation: Rotation::new(0., 0., 0.),
        }
    }
    pub fn to_matrix(&self) -> Matrix4 {
        Matrix4::translation(self.origin) * Matrix4::rotation(self.rotation)
    }
}

use cgmath::Vector3;
#[derive(Clone, Copy, Debug)]
pub struct Sun {
    pub phi: f32,
    pub theta: f32,
    /// radius in radians
    pub radius: f32,
}
impl Sun {
    pub fn make_direction_vector(&self) -> Vector3<f32> {
        let r = self.phi.cos();
        Vector3::new(r * self.theta.cos(), self.phi.sin(), r * self.theta.sin())
    }
}

use crate::prelude::RayScalar;
use cgmath::Vector3;
#[derive(Clone, Copy, Debug)]
pub struct Sun {
    pub phi: RayScalar,
    pub theta: RayScalar,
    /// radius in radians
    pub radius: RayScalar,
}
impl Sun {
    pub fn make_direction_vector(&self) -> Vector3<RayScalar> {
        let r = self.phi.cos();
        Vector3::new(r * self.theta.cos(), self.phi.sin(), r * self.theta.sin())
    }
}

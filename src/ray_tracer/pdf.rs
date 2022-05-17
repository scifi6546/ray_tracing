use crate::prelude::*;
use cgmath::{num_traits::FloatConst, InnerSpace, Vector3};
pub trait PDF {
    fn value(&self, direction: &Vector3<f32>) -> f32;
    fn generate(&self) -> Vector3<f32>;
}
pub struct CosinePdf {
    pub uvw: OrthoNormalBasis,
}
impl PDF for CosinePdf {
    fn value(&self, direction: &Vector3<f32>) -> f32 {
        let cos = direction.normalize().dot(self.uvw.w());

        if cos <= 0.0 {
            0.0
        } else {
            cos / f32::PI()
        }
    }

    fn generate(&self) -> Vector3<f32> {
        self.uvw.local(random_cosine_direction())
    }
}

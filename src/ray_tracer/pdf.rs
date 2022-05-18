use crate::prelude::*;
use crate::ray_tracer::World;
use cgmath::{num_traits::FloatConst, InnerSpace, Vector3};

pub trait PDF {
    fn value(&self, direction: &Vector3<f32>) -> f32;
    fn generate(&self, world: &World) -> (Vector3<f32>, f32);
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

    fn generate(&self, _world: &World) -> (Vector3<f32>, f32) {
        let direction = self.uvw.local(random_cosine_direction());
        let cos = direction.normalize().dot(self.uvw.w());

        let value = if cos <= 0.0 { 0.0 } else { cos / f32::PI() };
        (direction, value)
    }
}
pub struct LightPdf {}
impl PDF for LightPdf {
    fn value(&self, direction: &Vector3<f32>) -> f32 {
        todo!()
    }

    fn generate(&self, world: &World) -> (Vector3<f32>, f32) {
        let idx = rand_u32(0, world.lights.len() as u32) as usize;
        world.lights[idx].generate_ray_in_area(todo!("origin"), todo!("time"));
        todo!()
    }
}

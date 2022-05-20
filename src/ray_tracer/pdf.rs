use crate::prelude::*;
use crate::ray_tracer::World;
use cgmath::{num_traits::FloatConst, InnerSpace, Vector3};

pub trait PDF {
    //  fn value(&self, direction: &Vector3<f32>) -> f32;
    fn generate(&self, incoming_ray: Ray, world: &World) -> Option<(Vector3<f32>, f32)>;
}
pub struct CosinePdf {
    pub uvw: OrthoNormalBasis,
}
impl PDF for CosinePdf {
    fn generate(&self, _ray: Ray, _world: &World) -> Option<(Vector3<f32>, f32)> {
        let direction = self.uvw.local(random_cosine_direction());
        let cos = direction.normalize().dot(self.uvw.w());

        let value = if cos <= 0.0 { 0.0 } else { cos / f32::PI() };
        Some((direction, value))
    }
}
pub struct LightPdf {}
impl PDF for LightPdf {
    fn generate(&self, incoming_ray: Ray, world: &World) -> Option<(Vector3<f32>, f32)> {
        let idx = rand_u32(0, world.lights.len() as u32) as usize;
        let (ray, area, to_light) =
            world.lights[idx].generate_ray_in_area(incoming_ray.origin, incoming_ray.time);
        let light_cos = to_light.normalize().y.abs();
        let dist_squared = to_light.dot(to_light);
        if light_cos >= 0.000001 {
            let value = dist_squared / (light_cos * area);
            if debug() {
                println!(
                    "area: {}, light cos: {}, idx: {} ,value: {}, incoming ray: {}",
                    area, light_cos, idx, value, incoming_ray
                )
            }
            Some((ray.direction.normalize(), value))
        } else {
            None
        }
    }
}

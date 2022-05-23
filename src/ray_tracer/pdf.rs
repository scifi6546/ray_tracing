use crate::prelude::*;
use crate::ray_tracer::World;
use cgmath::{num_traits::FloatConst, InnerSpace, Point3, Vector3};

pub trait PDF {
    //  fn value(&self, direction: &Vector3<f32>) -> f32;
    fn generate(
        &self,
        incoming_ray: Ray,
        hit_point: Point3<f32>,
        world: &World,
    ) -> Option<(Vector3<f32>, f32)>;
}
pub struct CosinePdf {
    pub uvw: OrthoNormalBasis,
}
impl PDF for CosinePdf {
    fn generate(
        &self,
        _ray: Ray,
        _hit_point: Point3<f32>,
        _world: &World,
    ) -> Option<(Vector3<f32>, f32)> {
        let direction = self.uvw.local(random_cosine_direction()).normalize();
        let cos = direction.dot(self.uvw.w());
        let value = if cos <= 0.0 { 0.0 } else { cos / f32::PI() };

        if debug() {
            println!("value: {},direction: {:#?}", value, direction);
        }
        Some((direction, value))
    }
}
pub struct LightPdf {}
impl PDF for LightPdf {
    fn generate(
        &self,
        incoming_ray: Ray,
        hit_point: Point3<f32>,
        world: &World,
    ) -> Option<(Vector3<f32>, f32)> {
        let idx = rand_u32(0, world.lights.len() as u32) as usize;
        let (ray, area, to_light) =
            world.lights[idx].generate_ray_in_area(hit_point, incoming_ray.time);
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
pub struct PdfList {
    items: Vec<Box<dyn PDF>>,
}
impl PdfList {
    pub fn new(items: Vec<Box<dyn PDF>>) -> Self {
        assert!(items.len() >= 1);
        Self { items }
    }
}
impl PDF for PdfList {
    fn generate(
        &self,
        incoming_ray: Ray,
        hit_point: Point3<f32>,
        world: &World,
    ) -> Option<(Vector3<f32>, f32)> {
        let random_pdf = rand_u32(0, self.items.len() as u32) as usize;
        if let Some((out_direction, pdf)) =
            self.items[random_pdf].generate(incoming_ray, hit_point, world)
        {
            // Some((out_direction, pdf / self.items.len() as f32))
            Some((out_direction, pdf))
        } else {
            None
        }
    }
}

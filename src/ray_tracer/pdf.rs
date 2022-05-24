use crate::prelude::*;
use crate::ray_tracer::hittable::Hittable;
use crate::ray_tracer::World;
use cgmath::{num_traits::FloatConst, InnerSpace, Point3, Vector3};
use std::ops::Neg;
use std::rc::Rc;

pub trait PDF {
    fn value(&self, direction: &Ray, world: &World) -> f32;
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
impl CosinePdf {
    pub fn new(normal: Vector3<f32>) -> Self {
        Self {
            uvw: OrthoNormalBasis::build_from_w(normal),
        }
    }
}
impl PDF for CosinePdf {
    fn value(&self, ray: &Ray, _world: &World) -> f32 {
        let cos = ray.direction.dot(self.uvw.w());
        if cos <= 0.0 {
            0.0
        } else {
            cos / f32::PI()
        }
    }

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
    fn value(&self, ray: &Ray, world: &World) -> f32 {
        if let Some((light, hit)) = world.nearest_light_hit(ray, ray.time, f32::MAX) {
            let to_light = hit.position - ray.origin;
            let light_cos = to_light.normalize().dot(hit.normal).abs();
            let dist_squared = to_light.dot(to_light);

            if light_cos >= 0.000001 {
                let value = light.prob(Ray {
                    origin: hit.position,
                    direction: to_light.normalize(),
                    time: hit.t,
                });

                value
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    fn generate(
        &self,
        incoming_ray: Ray,
        hit_point: Point3<f32>,
        world: &World,
    ) -> Option<(Vector3<f32>, f32)> {
        if world.lights.is_empty() {
            return None;
        }
        let idx = rand_u32(0, world.lights.len() as u32) as usize;
        let area_info = world.lights[idx].generate_ray_in_area(hit_point, incoming_ray.time);

        let light_cos = area_info
            .to_area
            .direction
            .normalize()
            .dot(area_info.normal)
            .abs();
        let dist_squared = area_info.direction.dot(area_info.direction);
        if light_cos >= 0.000001 {
            let value = dist_squared / (light_cos * area_info.area);
            if debug() {
                println!(
                    "area: {}, light cos: {}, idx: {} ,value: {}, incoming ray: {}",
                    area_info.area, light_cos, idx, value, incoming_ray
                )
            }
            Some((area_info.to_area.direction.normalize(), value))
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
    fn value(&self, direction: &Ray, world: &World) -> f32 {
        todo!()
    }

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
            let total = world
                .lights
                .iter()
                .map(|light| {
                    light.prob(Ray {
                        origin: hit_point,
                        direction: out_direction,
                        time: 0.0,
                    })
                })
                .collect::<Vec<_>>();
            let avg: f32 = total.iter().sum::<f32>() / total.len() as f32;

            Some((out_direction, avg))
        } else {
            None
        }
    }
}
pub struct ScatterRecord {
    pub specular_ray: Option<Ray>,
    pub attenuation: RgbColor,
    pub pdf: Option<Rc<dyn PDF>>,
}

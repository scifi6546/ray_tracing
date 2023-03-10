use super::{hittable::Hittable, World};
use crate::prelude::*;

use crate::ray_tracer::hittable::HitRecord;
use cgmath::{num_traits::FloatConst, InnerSpace, Point3, Vector3};
use std::rc::Rc;

pub trait Pdf {
    fn value(&self, direction: &Ray, world: &World) -> f32;
    /// Checks if the PDF is valid for the given world
    fn is_valid(&self, world: &World) -> bool;
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
impl Pdf for CosinePdf {
    fn value(&self, ray: &Ray, _world: &World) -> f32 {
        let cos = ray.direction.dot(self.uvw.w());
        if cos <= 0.0 {
            0.0
        } else {
            cos / f32::PI()
        }
    }

    fn is_valid(&self, _world: &World) -> bool {
        true
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

        Some((direction, value))
    }
}
pub struct LightPdf {}
impl Pdf for LightPdf {
    fn value(&self, ray: &Ray, world: &World) -> f32 {
        if let Some((light, hit)) = world.nearest_light_hit(ray, ray.time, f32::MAX) {
            let to_light = hit.position - ray.origin;
            let light_cos = to_light.normalize().dot(hit.normal).abs();

            if light_cos >= 0.000001 {
                light.prob(Ray {
                    origin: ray.origin,
                    direction: to_light.normalize(),
                    time: hit.t,
                })
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
    fn is_valid(&self, world: &World) -> bool {
        !world.lights.is_empty()
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

            Some((area_info.to_area.direction.normalize(), value))
        } else {
            None
        }
    }
}
pub struct PdfList {
    items: Vec<Box<dyn Pdf>>,
}
impl PdfList {
    pub fn new(items: Vec<Box<dyn Pdf>>) -> Self {
        assert!(!items.is_empty());
        Self { items }
    }
}
impl Pdf for PdfList {
    fn value(&self, direction: &Ray, world: &World) -> f32 {
        let hit = self
            .items
            .iter()
            .filter(|item| item.is_valid(world))
            .map(|item| item.value(direction, world))
            .collect::<Vec<_>>();
        hit.iter().sum::<f32>() / hit.len() as f32
    }

    fn is_valid(&self, world: &World) -> bool {
        self.items
            .iter()
            .map(|item| item.is_valid(world))
            .any(|x| x)
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
            let mut sum = 0.0f32;
            let mut total = 0;
            let value_ray = Ray {
                origin: hit_point,
                direction: out_direction,
                time: 0.0,
            };
            for i in 0..self.items.len() {
                if i != random_pdf {
                    sum += self.items[i].value(&value_ray, world);
                    total += 1;
                }
            }
            if total != 0 {
                Some((out_direction, (sum + pdf) / (total + 1) as f32))
            } else {
                None
            }
        } else {
            None
        }
    }
}
#[derive(Clone)]
pub struct ScatterRecord {
    pub specular_ray: Option<Ray>,
    pub attenuation: RgbColor,
    pub pdf: Option<Rc<dyn Pdf>>,
    pub scattering_pdf: fn(Ray, &HitRecord, Ray) -> Option<f32>,
}

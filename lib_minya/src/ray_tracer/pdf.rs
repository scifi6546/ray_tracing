use super::{hittable::Hittable, World};
use crate::prelude::*;

use crate::ray_tracer::hittable::HitRecord;
use cgmath::{num_traits::FloatConst, InnerSpace, Point3, Vector3};
use std::{fmt, rc::Rc};

pub trait Pdf {
    fn value(&self, direction: &Ray, world: &World) -> Option<f32>;
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
    fn value(&self, ray: &Ray, _world: &World) -> Option<f32> {
        let cos = ray.direction.dot(self.uvw.w());
        if cos <= 0.0 {
            Some(0.0)
        } else {
            Some(cos / f32::PI())
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
    fn value(&self, ray: &Ray, world: &World) -> Option<f32> {
        if let Some((light, hit)) = world.nearest_light_hit(ray, ray.time, f32::MAX) {
            let to_light = hit.position - ray.origin;
            let light_cos = to_light.normalize().dot(hit.normal).abs();

            if light_cos >= 0.000001 {
                Some(light.prob(Ray {
                    origin: ray.origin,
                    direction: to_light.normalize(),
                    time: hit.t,
                }))
            } else {
                Some(0.0)
            }
        } else {
            None
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
pub struct SkyPDF {}
impl SkyPDF {}
impl Pdf for SkyPDF {
    fn value(&self, direction: &Ray, world: &World) -> Option<f32> {
        if world.sun.is_none() {
            None
        } else {
            let sun = world.sun.unwrap();
            let sun_vector = sun.make_direction_vector();
            let min_cos_distance = sun.radius.cos();
            if direction.direction.dot(sun_vector) >= min_cos_distance {
                Some(1.0)
            } else {
                Some(0.0)
            }
        }
    }

    fn is_valid(&self, world: &World) -> bool {
        todo!()
    }

    fn generate(
        &self,
        incoming_ray: Ray,
        hit_point: Point3<f32>,
        world: &World,
    ) -> Option<(Vector3<f32>, f32)> {
        /// generates theta and r inside of unit circle
        fn gen_unit_circle() -> (f32, f32) {
            let rand_r = rand_f32(0.0, 1.0);
            let rand_theta = rand_f32(0.0, 2.0 * f32::PI());
            (rand_r.sqrt(), rand_theta)
        }
        if world.sun.is_none() {
            return None;
        }
        let sun = world.sun.unwrap();

        let (r, theta) = gen_unit_circle();

        let r = r * sun.radius;
        let sun_vector = sun.make_direction_vector();
        let cross_vector = sun_vector.cross(Vector3::new(0.0, 0.0, 1.0));
        if cross_vector.magnitude() < 0.01 {
            panic!()
        }
        let k = cross_vector.normalize();
        let v_rot = r.cos() * sun_vector
            + (k.cross(sun_vector)) * theta.sin()
            + k * (k.dot(sun_vector)) * (1.0 - theta.cos());

        let area = f32::PI() * sun.radius.powi(2);
        if rand_u32(0, 1000) == 0 {
            // info!("area: {}, v_rot: {:#?}, k: {:#?}", area, v_rot, k);
        }
        let area_sphere = 4.0 * f32::PI();

        // rotation vector, https://en.wikipedia.org/wiki/Rodrigues'_rotation_formula
        //
        Some((v_rot, area / area_sphere))
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
    fn value(&self, direction: &Ray, world: &World) -> Option<f32> {
        let hit = self
            .items
            .iter()
            .filter(|item| item.is_valid(world))
            .filter_map(|item| item.value(direction, world))
            .collect::<Vec<_>>();
        Some(hit.iter().sum::<f32>() / hit.len() as f32)
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
                if let Some(value) = self.items[i].value(&value_ray, world) {
                    if i != random_pdf {
                        sum += value;
                        total += 1;
                    }
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
impl fmt::Debug for ScatterRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scatter Record")
            .field("specular_ray", &self.specular_ray)
            .field("attenuation", &self.attenuation)
            .field("pdf", if self.pdf.is_some() { &"Some" } else { &"None" })
            .finish()
    }
}

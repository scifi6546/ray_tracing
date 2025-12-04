use super::{hittable::Hittable, World};
use crate::prelude::*;

use crate::ray_tracer::hittable::HitRecord;
use crate::ray_tracer::rand_unit_vec;
use cgmath::{num_traits::FloatConst, InnerSpace, Point3, Vector3};

use std::{fmt, rc::Rc};

pub trait Pdf {
    fn value(&self, direction: &Ray, world: &World) -> Option<RayScalar>;
    /// Checks if the PDF is valid for the given world
    fn is_valid(&self, world: &World) -> bool;
    fn generate(
        &self,
        incoming_ray: Ray,
        hit_point: Point3<RayScalar>,
        world: &World,
    ) -> Option<(Vector3<RayScalar>, RayScalar)>;
}
pub struct CosinePdf {
    pub uvw: OrthoNormalBasis,
}
impl CosinePdf {
    pub fn new(normal: Vector3<RayScalar>) -> Self {
        Self {
            uvw: OrthoNormalBasis::build_from_w(normal),
        }
    }
}
impl Pdf for CosinePdf {
    fn value(&self, ray: &Ray, _world: &World) -> Option<RayScalar> {
        let cos = ray.direction.dot(self.uvw.w());
        if cos <= 0.0 {
            Some(0.0)
        } else {
            Some(cos / RayScalar::PI())
        }
    }

    fn is_valid(&self, _world: &World) -> bool {
        true
    }

    fn generate(
        &self,
        _ray: Ray,
        _hit_point: Point3<RayScalar>,
        _world: &World,
    ) -> Option<(Vector3<RayScalar>, RayScalar)> {
        let direction = self.uvw.local(random_cosine_direction()).normalize();
        let cos = direction.dot(self.uvw.w());
        let value = if cos <= 0.0 {
            0.0
        } else {
            cos / RayScalar::PI()
        };

        Some((direction, value))
    }
}
pub struct LightPdf {}
impl Pdf for LightPdf {
    fn value(&self, ray: &Ray, world: &World) -> Option<RayScalar> {
        if let Some((light, hit)) = world.nearest_light_hit(ray, ray.time, RayScalar::MAX) {
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
        hit_point: Point3<RayScalar>,
        world: &World,
    ) -> Option<(Vector3<RayScalar>, RayScalar)> {
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
pub struct SkyPdf {}
impl SkyPdf {}
impl Pdf for SkyPdf {
    fn value(&self, direction: &Ray, world: &World) -> Option<RayScalar> {
        if world.sun.is_none() {
            Some(1.0)
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

    fn is_valid(&self, _world: &World) -> bool {
        true
    }

    fn generate(
        &self,
        _incoming_ray: Ray,
        _hit_point: Point3<RayScalar>,
        world: &World,
    ) -> Option<(Vector3<RayScalar>, RayScalar)> {
        /// generates theta and r inside of unit circle
        fn gen_unit_circle() -> (RayScalar, RayScalar) {
            let rand_r = rand_scalar(0.0, 1.0);
            let rand_theta = rand_scalar(0.0, 2.0 * RayScalar::PI());
            (rand_r.sqrt(), rand_theta)
        }
        if world.sun.is_none() {
            let rand_vector = rand_unit_vec();
            Some((rand_vector, 4.0 * RayScalar::PI()))
        } else {
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

            let area = RayScalar::PI() * sun.radius.powi(2);

            let area_sphere = 4.0 * RayScalar::PI();

            // rotation vector, https://en.wikipedia.org/wiki/Rodrigues'_rotation_formula
            //
            Some((v_rot, area / area_sphere))
        }
    }
}

pub(crate) struct LambertianPDF {
    sin_pdf: CosinePdf,
    light_pdf: LightPdf,
    sky_pdf: SkyPdf,
}
impl LambertianPDF {
    pub fn new(normal: Vector3<RayScalar>) -> Self {
        Self {
            sin_pdf: CosinePdf::new(normal),
            light_pdf: LightPdf {},
            sky_pdf: SkyPdf {},
        }
    }
}
impl Pdf for LambertianPDF {
    fn value(&self, direction: &Ray, world: &World) -> Option<RayScalar> {
        let mut value = 0.0;
        let mut count = 0;
        for v in [
            self.sin_pdf.value(direction, world),
            self.light_pdf.value(direction, world),
            self.sky_pdf.value(direction, world),
        ]
        .iter()
        .filter_map(|v| *v)
        {
            value += v;
            count += 1;
        }
        Some(value / count as RayScalar)
    }

    fn is_valid(&self, world: &World) -> bool {
        self.sin_pdf.is_valid(world)
            && self.light_pdf.is_valid(world)
            && self.sky_pdf.is_valid(world)
    }

    fn generate(
        &self,
        incoming_ray: Ray,
        hit_point: Point3<RayScalar>,
        world: &World,
    ) -> Option<(Vector3<RayScalar>, RayScalar)> {
        let r = rand_u32(0, 3);
        let v = match r {
            0 => self.sin_pdf.generate(incoming_ray, hit_point, world),
            1 => self.light_pdf.generate(incoming_ray, hit_point, world),
            2 => self.sky_pdf.generate(incoming_ray, hit_point, world),
            _ => panic!(),
        };
        if v.is_some() {
            let (out_direction, pdf) = v.unwrap();
            let mut sum: RayScalar = 0.0;
            let mut total = 0;
            let value_ray = Ray {
                origin: hit_point,
                direction: out_direction,
                time: 0.0,
            };
            let values = match r {
                0 => [
                    self.light_pdf.value(&value_ray, world),
                    self.sky_pdf.value(&value_ray, world),
                ],
                1 => [
                    self.sin_pdf.value(&value_ray, world),
                    self.sky_pdf.value(&value_ray, world),
                ],
                2 => [
                    self.light_pdf.value(&value_ray, world),
                    self.sin_pdf.value(&value_ray, world),
                ],
                _ => panic!(),
            };
            for value in values.iter().flatten() {
                sum += value;
                total += 1;
            }
            if total != 0 {
                Some((out_direction, (sum + pdf) / (total + 1) as RayScalar))
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
    pub scattering_pdf: fn(Ray, &HitRecord, Ray) -> Option<RayScalar>,
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

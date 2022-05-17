use super::{rand_unit_vec, reflect, vec_near_zero, HitRecord, Ray, RgbColor, Texture};

use crate::prelude::{random_cosine_direction, OrthoNormalBasis};
use cgmath::{num_traits::*, prelude::*, InnerSpace, Point2, Point3, Vector2, Vector3};

pub type PDF = f32;
pub trait Material {
    fn scatter(&self, ray_in: Ray, record_in: &HitRecord) -> Option<(RgbColor, Ray, PDF)>;
    fn scattering_pdf(&self, ray_in: Ray, record_in: &HitRecord, scattered_ray: Ray) -> f32;
    fn emmit(&self, record: &HitRecord) -> RgbColor {
        RgbColor::new(0.0, 0.0, 0.0)
    }
}
pub struct Lambertian {
    pub albedo: Box<dyn Texture>,
}
impl Material for Lambertian {
    fn scatter(&self, ray_in: Ray, record_in: &HitRecord) -> Option<(RgbColor, Ray, PDF)> {
        let uvw = OrthoNormalBasis::build_from_w(record_in.normal);
        let direction = uvw.local(random_cosine_direction());

        let scatter_direction = record_in.normal + rand_unit_vec();

        let ray_out = if !vec_near_zero(scatter_direction) {
            Ray {
                origin: record_in.position,
                direction: scatter_direction,
                time: ray_in.time,
            }
        } else {
            Ray {
                origin: record_in.position,
                direction: record_in.normal,
                time: ray_in.time,
            }
        };
        let pdf = uvw.w().dot(ray_out.direction) / f32::PI();
        Some((
            self.albedo.color(record_in.uv, record_in.position),
            ray_out,
            pdf,
        ))
    }
    fn scattering_pdf(&self, ray_in: Ray, record_in: &HitRecord, scattered_ray: Ray) -> f32 {
        let cosine = record_in.normal.dot(scattered_ray.direction.normalize());
        if cosine < 0.0 {
            0.0
        } else {
            cosine / f32::PI()
        }
    }
}
pub struct Metal {
    pub albedo: Box<dyn Texture>,
    pub fuzz: f32,
}
impl Material for Metal {
    fn scatter(&self, ray_in: Ray, record_in: &HitRecord) -> Option<(RgbColor, Ray, PDF)> {
        let reflected = reflect(ray_in.direction.normalize(), record_in.normal);
        if reflected.dot(record_in.normal) > 0.0 {
            Some((
                self.albedo.color(record_in.uv, record_in.position),
                Ray {
                    origin: record_in.position,
                    direction: reflected + self.fuzz * rand_unit_vec(),
                    time: ray_in.time,
                },
                todo!(),
            ))
        } else {
            None
        }
    }

    fn scattering_pdf(&self, ray_in: Ray, record_in: &HitRecord, scattered_ray: Ray) -> f32 {
        todo!()
    }
}
pub struct Dielectric {
    pub index_refraction: f32,
    pub color: RgbColor,
}
impl Dielectric {
    fn refract(uv: Vector3<f32>, n: Vector3<f32>, etai_over_etat: f32) -> Vector3<f32> {
        let cos_theta = n.dot(-1.0 * uv).min(1.0);

        let r_out_perp = etai_over_etat * (uv + cos_theta * n);
        let r_out_parallel = -1.0 * n * (1.0 - (r_out_perp.dot(r_out_perp))).abs().sqrt();
        r_out_perp + r_out_parallel
    }
    fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
        let r0 = ((1.0 - ref_idx) / (1.0 + ref_idx)).powi(2);
        r0 + (1.0 - r0) * ((1.0 - cosine).powi(5))
    }
}
impl Material for Dielectric {
    fn scatter(&self, ray_in: Ray, record_in: &HitRecord) -> Option<(RgbColor, Ray, PDF)> {
        let refraction_ratio = if record_in.front_face {
            1.0 / self.index_refraction
        } else {
            self.index_refraction
        };
        let unit_direction = ray_in.direction.normalize();
        let cos_theta = record_in.normal.dot(-1.0 * unit_direction).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let can_not_refract = (refraction_ratio * sin_theta) > 1.0;
        let direction = if can_not_refract
            || Self::reflectance(cos_theta, refraction_ratio) > rand::random::<f32>()
        {
            reflect(unit_direction, record_in.normal)
        } else {
            Self::refract(unit_direction, record_in.normal, refraction_ratio)
        };

        Some((
            self.color,
            Ray {
                origin: record_in.position,
                direction,
                time: ray_in.time,
            },
            todo!(),
        ))
    }

    fn scattering_pdf(&self, ray_in: Ray, record_in: &HitRecord, scattered_ray: Ray) -> f32 {
        todo!()
    }
}
pub struct DiffuseLight {
    pub emit: Box<dyn Texture>,
}
impl Material for DiffuseLight {
    fn scatter(&self, ray_in: Ray, record_in: &HitRecord) -> Option<(RgbColor, Ray, PDF)> {
        None
    }

    fn scattering_pdf(&self, ray_in: Ray, record_in: &HitRecord, scattered_ray: Ray) -> f32 {
        0.0
    }

    fn emmit(&self, record: &HitRecord) -> RgbColor {
        if record.front_face {
            self.emit.color(record.uv, record.position)
        } else {
            RgbColor::new(0.0, 0.0, 0.0)
        }
    }
}
pub struct Isotropic {
    pub albedo: Box<dyn Texture>,
}

impl Material for Isotropic {
    fn scatter(&self, ray_in: Ray, record_in: &HitRecord) -> Option<(RgbColor, Ray, PDF)> {
        Some((
            self.albedo.color(record_in.uv, record_in.position),
            Ray {
                origin: record_in.position,
                direction: rand_unit_vec(),
                time: ray_in.time,
            },
            todo!(),
        ))
    }

    fn scattering_pdf(&self, ray_in: Ray, record_in: &HitRecord, scattered_ray: Ray) -> f32 {
        todo!()
    }
}

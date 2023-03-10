use super::{
    rand_unit_vec, reflect, CosinePdf, HitRay, HitRecord, LightPdf, PdfList, Ray, RgbColor,
    ScatterRecord, Texture,
};
use cgmath::{num_traits::*, InnerSpace, Vector3};
use dyn_clone::{clone_box, DynClone};
use std::ops::Deref;

use std::rc::Rc;

//pub type PDF = f32;
pub trait Material: Send + DynClone {
    fn name(&self) -> &'static str;
    fn scatter(&self, ray_in: Ray, record_in: &HitRay) -> Option<ScatterRecord>;
    fn scattering_pdf(&self, ray_in: Ray, record_in: &HitRecord, scattered_ray: Ray)
        -> Option<f32>;
    fn emmit(&self, _record: &HitRay) -> Option<RgbColor> {
        None
    }
}

pub struct Lambertian {
    pub albedo: Box<dyn Texture>,
}
impl Lambertian {
    fn scattering_pdf_fn(_ray_in: Ray, record_in: &HitRecord, scattered_ray: Ray) -> Option<f32> {
        let cosine = record_in.normal.dot(scattered_ray.direction.normalize());
        if cosine < 0.0 {
            None
        } else {
            Some(cosine / f32::PI())
        }
    }
}
impl Clone for Lambertian {
    fn clone(&self) -> Self {
        Self {
            albedo: clone_box(self.albedo.deref()),
        }
    }
}
impl Material for Lambertian {
    fn name(&self) -> &'static str {
        "Lambertian"
    }
    fn scatter(&self, _ray_in: Ray, record_in: &HitRay) -> Option<ScatterRecord> {
        let attenuation = self.albedo.color(record_in.uv, record_in.position);

        let scatter_record = ScatterRecord {
            specular_ray: None,
            attenuation,
            pdf: Some(Rc::new(PdfList::new(vec![
                Box::new(CosinePdf::new(record_in.normal)),
                Box::new(LightPdf {}),
            ]))),
            scattering_pdf: Self::scattering_pdf_fn,
        };
        Some(scatter_record)
    }
    fn scattering_pdf(
        &self,
        _ray_in: Ray,
        record_in: &HitRecord,
        scattered_ray: Ray,
    ) -> Option<f32> {
        let cosine = record_in.normal.dot(scattered_ray.direction.normalize());
        if cosine < 0.0 {
            None
        } else {
            Some(cosine / f32::PI())
        }
    }
}

pub struct Metal {
    pub albedo: Box<dyn Texture>,
    pub fuzz: f32,
}
impl Metal {
    fn scattering_pdf_fn(_ray_in: Ray, record_in: &HitRecord, scattered_ray: Ray) -> Option<f32> {
        panic!("material is specular")
    }
}
impl Clone for Metal {
    fn clone(&self) -> Self {
        Self {
            albedo: clone_box(self.albedo.deref()),
            fuzz: self.fuzz,
        }
    }
}
impl Material for Metal {
    fn name(&self) -> &'static str {
        "Metal"
    }
    fn scatter(&self, ray_in: Ray, record_in: &HitRay) -> Option<ScatterRecord> {
        let reflected = reflect(ray_in.direction.normalize(), record_in.normal);

        if reflected.dot(record_in.normal) > 0.0 {
            let out_ray = Ray {
                origin: record_in.position,
                direction: reflected + self.fuzz * rand_unit_vec(),
                time: ray_in.time,
            };

            Some(ScatterRecord {
                specular_ray: Some(out_ray),
                attenuation: self.albedo.color(record_in.uv, record_in.position),
                pdf: None,
                scattering_pdf: Self::scattering_pdf_fn,
            })
        } else {
            None
        }
    }

    fn scattering_pdf(
        &self,
        _ray_in: Ray,
        _record_in: &HitRecord,
        _scattered_ray: Ray,
    ) -> Option<f32> {
        panic!("material is specular")
    }
}
#[derive(Clone)]
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
    fn scattering_pdf_fn(_ray_in: Ray, _record_in: &HitRecord, _scattered_ray: Ray) -> Option<f32> {
        panic!("material is specular should not have scattering")
    }
}
impl Material for Dielectric {
    fn name(&self) -> &'static str {
        "Dielectric"
    }
    fn scatter(&self, ray_in: Ray, record_in: &HitRay) -> Option<ScatterRecord> {
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

        Some(ScatterRecord {
            specular_ray: Some(Ray {
                origin: record_in.position,
                direction,
                time: ray_in.time,
            }),
            attenuation: self.color,
            pdf: None,
            scattering_pdf: Self::scattering_pdf_fn,
        })
    }

    fn scattering_pdf(
        &self,
        _ray_in: Ray,
        _record_in: &HitRecord,
        _scattered_ray: Ray,
    ) -> Option<f32> {
        panic!("material is specular should not have scattering")
    }
}

pub struct DiffuseLight {
    pub emit: Box<dyn Texture>,
}
impl DiffuseLight {}
impl Clone for DiffuseLight {
    fn clone(&self) -> Self {
        Self {
            emit: clone_box(self.emit.deref()),
        }
    }
}
impl Material for DiffuseLight {
    fn name(&self) -> &'static str {
        "Diffuse Light"
    }
    fn scatter(&self, _ray_in: Ray, _record_in: &HitRay) -> Option<ScatterRecord> {
        None
    }

    fn scattering_pdf(
        &self,
        _ray_in: Ray,
        _record_in: &HitRecord,
        _scattered_ray: Ray,
    ) -> Option<f32> {
        panic!("should not have scattering")
    }

    fn emmit(&self, record: &HitRay) -> Option<RgbColor> {
        if record.front_face {
            Some(self.emit.color(record.uv, record.position))
        } else {
            Some(RgbColor::new(0.0, 0.0, 0.0))
        }
    }
}

pub struct Isotropic {
    pub albedo: Box<dyn Texture>,
}
impl Isotropic {
    fn scattering_pdf_fn(_ray_in: Ray, _record_in: &HitRecord, _scattered_ray: Ray) -> Option<f32> {
        panic!("should not have scattering")
    }
}
impl Clone for Isotropic {
    fn clone(&self) -> Self {
        Self {
            albedo: clone_box(self.albedo.deref()),
        }
    }
}
impl Material for Isotropic {
    fn name(&self) -> &'static str {
        "Isotropic"
    }
    fn scatter(&self, ray_in: Ray, record_in: &HitRay) -> Option<ScatterRecord> {
        Some(ScatterRecord {
            specular_ray: Some(Ray {
                origin: record_in.position,
                direction: rand_unit_vec(),
                time: ray_in.time,
            }),
            attenuation: self.albedo.color(record_in.uv, record_in.position),
            pdf: None,
            scattering_pdf: Self::scattering_pdf_fn,
        })
    }

    fn scattering_pdf(
        &self,
        _ray_in: Ray,
        _record_in: &HitRecord,
        _scattered_ray: Ray,
    ) -> Option<f32> {
        panic!("should not have scattering")
    }
}

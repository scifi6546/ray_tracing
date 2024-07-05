use super::Leafable;

use base_lib::RgbColor;
use std::rc::Rc;

use crate::{
    prelude::Ray,
    ray_tracer::{
        hittable::{HitRay, HitRecord},
        material::Material,
        pdf::{LambertianPDF, ScatterRecord},
    },
};
use cgmath::{num_traits::FloatConst, prelude::*};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VoxelMaterial {
    pub color: RgbColor,
}
impl Leafable for VoxelMaterial {}
impl VoxelMaterial {
    fn scattering_pdf_fn(_ray_in: Ray, record_in: &HitRecord, scattered_ray: Ray) -> Option<f32> {
        let cosine = record_in.normal.dot(scattered_ray.direction.normalize());
        if cosine < 0.0 {
            None
        } else {
            Some(cosine / f32::PI())
        }
    }
}
impl Material for VoxelMaterial {
    fn name(&self) -> &'static str {
        "Voxel Material"
    }

    fn scatter(&self, _ray_in: Ray, record_in: &HitRay) -> Option<ScatterRecord> {
        Some(ScatterRecord {
            specular_ray: None,
            attenuation: self.color,
            pdf: Some(Rc::new(LambertianPDF::new(record_in.normal()))),
            scattering_pdf: Self::scattering_pdf_fn,
        })
    }

    fn scattering_pdf(
        &self,
        _ray_in: Ray,
        _record_in: &HitRecord,
        _scattered_ray: Ray,
    ) -> Option<f32> {
        None
    }
}

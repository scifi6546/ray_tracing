use super::super::{HitRay, HitRecord, Material, ScatterRecord};
use crate::{
    prelude::{Ray, RayScalar, RgbColor},
    ray_tracer::{
        pdf::{IsotropicPdf, LambertianPDF},
        rand_unit_vec,
    },
    reflect,
};
use cgmath::{num_traits::FloatConst, prelude::*};
use std::rc::Rc;
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Voxel {
    Solid(SolidVoxel),
    Volume(VolumeVoxel),
}
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum SolidVoxel {
    Lambertian { albedo: RgbColor },
    Reflect { albedo: RgbColor, fuzz: f32 },
}
impl SolidVoxel {
    pub fn to_material(self) -> VoxelMaterial {
        match self {
            Self::Lambertian { albedo: color } => VoxelMaterial::Lambertian { color },
            Self::Reflect { albedo, fuzz } => VoxelMaterial::Reflect { albedo, fuzz },
        }
    }
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VolumeEdgeEffect {
    None,
    Solid {
        hit_probability: f32,
        solid_material: SolidVoxel,
    },
}
#[derive(Copy, Clone, Debug)]
pub struct VolumeVoxel {
    pub density: RayScalar,
    pub color: RgbColor,
    pub edge_effect: VolumeEdgeEffect,
}
impl PartialEq for VolumeVoxel {
    fn eq(&self, rhs: &VolumeVoxel) -> bool {
        const VOLUME_ERROR_MARGIN: RayScalar = 0.0001;
        const COLOR_ERROR_MARGIN: f32 = 0.0001;
        (self.density - rhs.density).abs() < VOLUME_ERROR_MARGIN
            && (self.color.distance(rhs.color)) < COLOR_ERROR_MARGIN
            && self.edge_effect == rhs.edge_effect
    }
}
impl VolumeVoxel {
    pub(crate) fn volume_material(&self) -> VoxelMaterial {
        VoxelMaterial::Volume { color: self.color }
    }
}
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum VoxelMaterial {
    Lambertian { color: RgbColor },
    Reflect { albedo: RgbColor, fuzz: f32 },
    Volume { color: RgbColor },
}
impl VoxelMaterial {
    fn scattering_pdf_fn(
        _ray_in: Ray,
        record_in: &HitRecord,
        scattered_ray: Ray,
    ) -> Option<RayScalar> {
        let cosine = record_in.normal.dot(scattered_ray.direction.normalize());
        if cosine < 0.0 {
            None
        } else {
            Some(cosine / RayScalar::PI())
        }
    }
}
impl Material for VoxelMaterial {
    fn name(&self) -> &'static str {
        "Voxel Material"
    }

    fn scatter(&self, ray_in: Ray, record_in: &HitRay) -> Option<ScatterRecord> {
        match self {
            Self::Lambertian { color } => Some(ScatterRecord {
                specular_ray: None,
                attenuation: *color,
                pdf: Some(Rc::new(LambertianPDF::new(record_in.normal()))),
                scattering_pdf: Self::scattering_pdf_fn,
            }),
            Self::Volume { color, .. } => Some(ScatterRecord {
                specular_ray: None,
                attenuation: *color,
                pdf: Some(Rc::new(IsotropicPdf {})),
                scattering_pdf: Self::scattering_pdf_fn,
            }),
            Self::Reflect { albedo, fuzz } => {
                let reflected = reflect(ray_in.direction.normalize(), record_in.normal());
                if reflected.dot(record_in.normal()) > 0.0 {
                    let out_ray = Ray {
                        origin: record_in.position(),
                        direction: reflected + *fuzz as f64 * rand_unit_vec(),
                        time: ray_in.time,
                    };

                    Some(ScatterRecord {
                        specular_ray: Some(out_ray),
                        attenuation: *albedo,
                        pdf: None,
                        scattering_pdf: Self::scattering_pdf_fn,
                    })
                } else {
                    None
                }
            }
        }
    }

    fn scattering_pdf(
        &self,
        _ray_in: Ray,
        _record_in: &HitRecord,
        _scattered_ray: Ray,
    ) -> Option<RayScalar> {
        None
    }
}

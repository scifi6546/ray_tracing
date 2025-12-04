use super::Leafable;

use base_lib::RgbColor;
use std::rc::Rc;

use crate::prelude::RayScalar;
use crate::{
    prelude::Ray,
    ray_tracer::{
        hittable::{HitRay, HitRecord},
        material::Material,
        pdf::{LambertianPDF, ScatterRecord},
    },
};
use cgmath::{num_traits::FloatConst, prelude::*};

#[derive(Copy, Clone, Debug)]
pub enum VoxelMaterial {
    Solid { color: RgbColor },
    Empty,
}
impl PartialEq for VoxelMaterial {
    fn eq(&self, other: &Self) -> bool {
        const ERROR_MARGIN: f32 = 0.0001;
        match self {
            Self::Solid { color } => match other {
                Self::Solid { color: other_color } => {
                    (color.red - other_color.red).abs()
                        + (color.green - other_color.green).abs()
                        + (color.blue - other_color.blue).abs()
                        < ERROR_MARGIN
                }
                Self::Empty => false,
            },
            Self::Empty => match other {
                Self::Empty => true,
                Self::Solid { .. } => false,
            },
        }
    }
}
impl Eq for VoxelMaterial {}
impl Leafable for VoxelMaterial {
    fn is_solid(&self) -> bool {
        match self {
            Self::Solid { .. } => true,
            Self::Empty => false,
        }
    }
    fn empty() -> Self {
        Self::Empty
    }
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

    fn scatter(&self, _ray_in: Ray, record_in: &HitRay) -> Option<ScatterRecord> {
        match self {
            Self::Solid { color } => Some(ScatterRecord {
                specular_ray: None,
                attenuation: *color,
                pdf: Some(Rc::new(LambertianPDF::new(record_in.normal()))),
                scattering_pdf: Self::scattering_pdf_fn,
            }),
            Self::Empty => panic!("should never scatter here"),
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
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn is_same() {
        let c1 = VoxelMaterial::Solid {
            color: RgbColor {
                red: 0.5,
                green: 0.5,
                blue: 0.5,
            },
        };
        let c2 = VoxelMaterial::Solid {
            color: RgbColor {
                red: 0.5,
                green: 0.5,
                blue: 0.5,
            },
        };
        assert_eq!(c1, c2);
        assert_eq!(c2, c1);
    }
    #[test]
    fn empty_same() {
        let c1 = VoxelMaterial::Empty;
        let c2 = VoxelMaterial::Empty;
        assert_eq!(c1, c2);
        assert_eq!(c2, c1);
    }
    #[test]
    fn solid_empty_different() {
        let c1 = VoxelMaterial::Empty;
        let c2 = VoxelMaterial::Solid {
            color: RgbColor {
                red: 0.5,
                green: 0.5,
                blue: 0.5,
            },
        };
        assert_ne!(c1, c2);
        assert_ne!(c2, c1);
    }
}

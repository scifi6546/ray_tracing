use super::{HitType, Leafable};

use std::rc::Rc;

use crate::prelude::RayScalar;
use crate::{
    prelude::{Ray, RgbColor},
    ray_tracer::{
        hittable::{HitRay, HitRecord},
        material::Material,
        pdf::{IsotropicPdf, LambertianPDF, ScatterRecord},
        rand_unit_vec,
    },
    reflect,
};
use cgmath::{num_traits::FloatConst, prelude::*};
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VolumeEdgeEffect {
    None,
    Solid {
        hit_probability: f32,
        solid_material: SolidVoxel,
    },
}

#[derive(Copy, Clone, Debug)]
pub enum SolidVoxel {
    Lambertian { albedo: RgbColor },
    Reflect { albedo: RgbColor, fuzz: f32 },
}
impl PartialEq for SolidVoxel {
    fn eq(&self, rhs: &SolidVoxel) -> bool {
        const ERROR_MARGIN: f32 = 0.0001;
        match self {
            Self::Lambertian { albedo: color } => match rhs {
                Self::Lambertian {
                    albedo: other_color,
                } => color.distance(*other_color) < ERROR_MARGIN,
                Self::Reflect { .. } => false,
            },
            Self::Reflect { albedo, fuzz } => match rhs {
                Self::Lambertian { .. } => false,
                Self::Reflect {
                    albedo: rhs_albedo,
                    fuzz: rhs_fuzz,
                } => (albedo.distance(*rhs_albedo) + (fuzz - rhs_fuzz).abs()) < ERROR_MARGIN,
            },
        }
    }
}
impl SolidVoxel {
    pub(crate) fn to_material(self) -> VoxelMaterial {
        match self {
            Self::Lambertian { albedo: color } => VoxelMaterial::Lambertian { color },
            Self::Reflect { albedo, fuzz } => VoxelMaterial::Reflect { albedo, fuzz },
        }
    }
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
#[derive(Copy, Clone, Debug)]
pub enum Voxel {
    Solid(SolidVoxel),
    Volume(VolumeVoxel),
    Empty,
}
impl PartialEq for Voxel {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Solid(self_solid_material) => match other {
                Self::Solid(other_solid_material) => self_solid_material == other_solid_material,
                Self::Empty => false,
                Self::Volume { .. } => false,
            },
            Self::Empty => match other {
                Self::Empty => true,
                Self::Solid { .. } => false,
                Self::Volume { .. } => false,
            },
            Self::Volume(self_volume_material) => match other {
                Self::Solid { .. } => false,
                Self::Volume(other_volume_material) => {
                    self_volume_material == other_volume_material
                }
                Self::Empty => false,
            },
        }
    }
}
impl Eq for Voxel {}
impl Leafable for Voxel {
    type Material = VoxelMaterial;
    fn hit_type(&self) -> HitType {
        match self {
            Self::Solid { .. } => HitType::Solid,
            Self::Volume { .. } => HitType::Volume,
            Self::Empty => HitType::Empty,
        }
    }
    fn empty() -> Self {
        Self::Empty
    }
}
#[derive(Copy, Clone, Debug, PartialEq)]
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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn is_same() {
        let c1 = Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor {
                red: 0.5,
                green: 0.5,
                blue: 0.5,
            },
        });

        let c2 = Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor {
                red: 0.5,
                green: 0.5,
                blue: 0.5,
            },
        });
        assert_eq!(c1, c2);
        assert_eq!(c2, c1);
    }
    #[test]
    fn empty_same() {
        let c1 = Voxel::Empty;
        let c2 = Voxel::Empty;
        assert_eq!(c1, c2);
        assert_eq!(c2, c1);
    }
    #[test]
    fn solid_empty_different() {
        let c1 = Voxel::Empty;

        let c2 = Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor {
                red: 0.5,
                green: 0.5,
                blue: 0.5,
            },
        });
        assert_ne!(c1, c2);
        assert_ne!(c2, c1);
    }
    #[test]
    fn solid_lamb_metal() {
        let sl = Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor {
                red: 0.5,
                green: 0.5,
                blue: 0.5,
            },
        });
        let sm1 = Voxel::Solid(SolidVoxel::Reflect {
            albedo: RgbColor {
                red: 0.5,
                green: 0.5,
                blue: 0.5,
            },
            fuzz: 0.8,
        });
        let sm2 = Voxel::Solid(SolidVoxel::Reflect {
            albedo: RgbColor {
                red: 0.5,
                green: 0.5,
                blue: 0.5,
            },
            fuzz: 0.7,
        });
        let sm3 = Voxel::Solid(SolidVoxel::Reflect {
            albedo: RgbColor {
                red: 0.5,
                green: 0.5,
                blue: 0.2,
            },
            fuzz: 0.8,
        });
        let arr = [sl, sm1, sm2, sm3];
        for i in 0..arr.len() {
            for j in 0..arr.len() {
                if i == j {
                    assert_eq!(arr[i], arr[j])
                } else {
                    assert_ne!(arr[i], arr[j])
                }
            }
        }
    }
    #[test]
    fn volume_density() {
        let v1 = Voxel::Volume(VolumeVoxel {
            density: 0.5,
            color: RgbColor::WHITE,
            edge_effect: VolumeEdgeEffect::None,
        });
        let v2 = Voxel::Volume(VolumeVoxel {
            density: 0.8,
            color: RgbColor::WHITE,
            edge_effect: VolumeEdgeEffect::None,
        });
        assert_eq!(v1, v1);
        assert_eq!(v2, v2);
        assert_ne!(v1, v2);
        assert_ne!(v2, v1);
    }
    #[test]
    fn volume_color() {
        let v1 = Voxel::Volume(VolumeVoxel {
            density: 0.5,
            color: RgbColor::WHITE,
            edge_effect: VolumeEdgeEffect::None,
        });
        let v2 = Voxel::Volume(VolumeVoxel {
            density: 0.5,
            color: RgbColor {
                red: 1.0,
                green: 0.,
                blue: 0.,
            },
            edge_effect: VolumeEdgeEffect::None,
        });
        assert_eq!(v1, v1);
        assert_eq!(v2, v2);
        assert_ne!(v1, v2);
        assert_ne!(v2, v1);
    }
    #[test]
    fn volume_others() {
        let v = Voxel::Volume(VolumeVoxel {
            density: 0.5,
            color: RgbColor::WHITE,
            edge_effect: VolumeEdgeEffect::None,
        });
        let s = Voxel::Solid(SolidVoxel::Lambertian {
            albedo: RgbColor {
                red: 0.2,
                green: 0.6,
                blue: 0.8,
            },
        });
        let e = Voxel::Empty;
        assert_eq!(v, v);
        assert_eq!(s, s);
        assert_eq!(e, e);

        assert_ne!(v, s);
        assert_ne!(s, v);

        assert_ne!(v, e);
        assert_ne!(e, v);
    }
    #[test]
    fn volume_edge() {
        let vl = Voxel::Volume(VolumeVoxel {
            density: 0.5,
            color: RgbColor::WHITE,
            edge_effect: VolumeEdgeEffect::Solid {
                hit_probability: 0.2,
                solid_material: SolidVoxel::Lambertian {
                    albedo: RgbColor::WHITE,
                },
            },
        });
        let vl2 = Voxel::Volume(VolumeVoxel {
            density: 0.5,
            color: RgbColor::WHITE,
            edge_effect: VolumeEdgeEffect::Solid {
                hit_probability: 0.5,
                solid_material: SolidVoxel::Lambertian {
                    albedo: RgbColor::WHITE,
                },
            },
        });
        let vn = Voxel::Volume(VolumeVoxel {
            density: 0.5,
            color: RgbColor::WHITE,
            edge_effect: VolumeEdgeEffect::None,
        });
        assert_eq!(vl, vl);
        assert_ne!(vl, vn);
        assert_ne!(vl, vl2);

        assert_eq!(vn, vn);
        assert_ne!(vn, vl);
        assert_ne!(vn, vl2);

        assert_eq!(vl2, vl2);
        assert_ne!(vl2, vl);
        assert_ne!(vl2, vn);
    }
    #[test]
    fn volume_material() {
        assert_eq!(
            Voxel::Volume(VolumeVoxel {
                density: 0.2,
                color: RgbColor::WHITE,
                edge_effect: VolumeEdgeEffect::None,
            })
            .hit_type(),
            HitType::Volume
        );
        assert_eq!(
            Voxel::Solid(SolidVoxel::Lambertian {
                albedo: RgbColor::WHITE
            })
            .hit_type(),
            HitType::Solid
        );
        assert_eq!(Voxel::Empty.hit_type(), HitType::Empty);
    }
}

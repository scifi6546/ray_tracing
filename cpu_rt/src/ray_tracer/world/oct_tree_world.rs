mod load_model;
pub mod metal;

use super::{
    Camera, CameraInfo, ConstantColor, DiffuseLight, OctTree, SolidColor, Sphere, Transform, Voxel,
    WorldInfo,
};
pub use load_model::load_voxel_model;

use crate::{
    prelude::*,
    ray_tracer::hittable::{Object, SolidVoxel},
};

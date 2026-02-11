pub mod compare_voxel_world;
mod load_model;
pub mod metal;
pub mod volume;
use super::{
    Camera, CameraInfo, ConstantColor, DiffuseLight, OctTree, SolidColor, Sphere, Transform, Voxel,
    WorldInfo,
};
pub use load_model::load_voxel_model;

use crate::{
    prelude::*,
    ray_tracer::hittable::{Object, SolidVoxel},
};

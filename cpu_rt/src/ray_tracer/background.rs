mod constant_color;
pub(crate) mod saver_loader;
mod sky;
mod sun_sky;

use crate::{
    prelude::*,
    ray_tracer::save_file::{
        traits::{DynSavable, Savable},
        SceneSaveError,
    },
    Ray,
};

pub(crate) use constant_color::ConstantColor;
use dyn_clone::DynClone;
pub(crate) use sky::Sky;
pub(crate) use sun_sky::SunSky;

pub(crate) trait Background: Send + Sync + DynClone + DynSavable {
    fn color(&self, ray: Ray) -> RgbColor;
}

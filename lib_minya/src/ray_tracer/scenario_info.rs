use super::world::WorldInfo;
use crate::prelude::RayScalar;

pub(crate) enum LoadScenario {
    None,
    Prebuilt(String),
    Custom(WorldInfo),
}

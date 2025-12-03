use super::world::WorldInfo;

pub(crate) enum LoadScenario {
    None,
    Prebuilt(String),
    Custom(WorldInfo),
}

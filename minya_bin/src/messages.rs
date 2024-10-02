use lib_minya::ray_tracer::{ray_tracer_info::EntityField, CurrentShader};

/// Messages that the gui sends to the ray tracer.
pub enum GuiPushMessage {
    LoadScenario(String),
    SaveFile(std::path::PathBuf),
    SetShader(CurrentShader),
    SetEntityData,
    SetCameraData((String, EntityField)),
}

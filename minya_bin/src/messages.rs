use lib_minya::ray_tracer::{
    ray_tracer_info::{EntityField, RayTracerInfo},
    CurrentShader,
};

/// Messages that the gui sends to the ray tracer.
#[derive(Clone, Debug, PartialEq)]
pub enum GuiPushMessage {
    LoadScenario(String),
    SaveFile(std::path::PathBuf),
    SetShader(CurrentShader),
    SetCameraData((String, EntityField)),
    SetEntityInfo {
        entity_index: usize,
        field_name: String,
        field_value: EntityField,
    },
}
/// Messages that are sent from the ray tracer to the GUI.
#[derive(Clone, Debug, PartialEq)]
pub enum GuiSendMessage {
    UpdateRayTracerInfo(RayTracerInfo),
}

use super::Camera;
use crate::prelude::RayScalar;
use cgmath::{Point3, Vector3};
use log::error;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct RayTracerInfo {
    pub scenarios: Vec<ScenarioInfo>,
    pub loaded_entities: WorldEntityCollection,
}
#[derive(Clone, Debug, PartialEq, Ord, PartialOrd, Eq)]
pub struct ScenarioInfo {
    pub name: String,
}
#[derive(Clone, Debug, PartialEq)]
pub struct WorldEntityCollection {
    pub main_camera: Camera,
    pub entities: Vec<EntityInfo>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct EntityInfo {
    pub fields: HashMap<String, EntityField>,
    pub name: String,
}

pub trait Entity {
    fn name(&self) -> String;
    fn fields(&self) -> HashMap<String, EntityField> {
        HashMap::new()
    }
    fn set_field(&mut self, _key: String, _value: EntityField) {
        error!("No Entity fields setter defined")
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum EntityField {
    Point3(Point3<RayScalar>),
    Angle(Vector3<RayScalar>),
    Float(RayScalar),
}

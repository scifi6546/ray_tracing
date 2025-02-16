use super::{Background, Savable, SceneSaveError};
use crate::prelude::Ray;
use base_lib::RgbColor;
use rusqlite::Connection;
use uuid::Uuid;

#[derive(Clone)]
pub struct ConstantColor {
    pub color: RgbColor,
}
impl Background for ConstantColor {
    fn color(&self, _ray: Ray) -> RgbColor {
        self.color
    }
}
impl Default for ConstantColor {
    fn default() -> Self {
        Self {
            color: RgbColor::WHITE,
        }
    }
}
impl Savable for ConstantColor {
    fn database_name() -> &'static str {
        "constant_color"
    }

    fn make_schema(connection: &Connection) -> Result<(), SceneSaveError> {
        let name = <Self as Savable>::database_name();
        let sql_call = format!(
            "CREATE TABLE {}({}_id blob PRIMARY KEY, red double, green double, blue double);",
            name, name
        );
        connection.execute(&sql_call, ())?;
        Ok(())
    }

    fn delete_schema(connection: &mut Connection) {
        todo!()
    }

    fn save(&self, connection: &Connection) -> Result<Uuid, SceneSaveError> {
        todo!()
    }

    fn load(id: Uuid, connection: &Connection) -> Result<Vec<Self>, SceneSaveError> {
        todo!("load constant color")
    }
}

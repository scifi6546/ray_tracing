use super::{Background, Savable, SceneSaveError};
use crate::prelude::{Ray, RayScalar, RgbColor};
use crate::ray_tracer::save_file::traits::DynSavable;
use cgmath::InnerSpace;
use rusqlite::Connection;
use uuid::Uuid;

#[derive(Clone)]
pub struct Sky {
    pub intensity: RayScalar,
}
impl Background for Sky {
    fn color(&self, ray: Ray) -> RgbColor {
        let unit = ray.direction.normalize();
        let t = 0.5 * (unit.y + 1.0);
        let color = (1.0 - t)
            * RgbColor {
                red: 1.0,
                blue: 1.0,
                green: 1.0,
            }
            + t * RgbColor {
                red: 0.5,
                green: 0.7,
                blue: 1.0,
            };
        self.intensity * color
    }
}
impl Default for Sky {
    fn default() -> Self {
        Self { intensity: 1.0 }
    }
}

impl Savable for Sky {
    fn database_name() -> &'static str {
        "sky"
    }

    fn make_schema(connection: &Connection) -> Result<(), SceneSaveError> {
        let name = <Self as Savable>::database_name();
        let sql_call = format!(
            "CREATE TABLE {}({}_id blob PRIMARY KEY, intensity double);",
            name, name
        );
        connection.execute(&sql_call, ())?;
        Ok(())
    }

    fn delete_schema(_connection: &mut Connection) {
        todo!()
    }

    fn save(&self, connection: &Connection) -> Result<Uuid, SceneSaveError> {
        let id = Uuid::new_v4();
        let statement = format!(
            "INSERT INTO {}({}_id, intensity) VALUES (?1,?2);",
            self.database_name(),
            self.database_name()
        );
        connection.execute(&statement, (id, self.intensity))?;
        Ok(id)
    }
    fn load(id: Uuid, connection: &Connection) -> Result<Vec<Self>, SceneSaveError> {
        let statement = format!(
            "SELECT intensity FROM {} WHERE {}_id = ?1",
            <Self as Savable>::database_name(),
            <Self as Savable>::database_name()
        );
        let mut query = connection.prepare(&statement)?;
        let output = Ok(query
            .query_map([id], |row| {
                Ok(Self {
                    intensity: row.get(0)?,
                })
            })?
            .filter_map(|v| v.ok())
            .collect::<Vec<_>>());
        output
    }
}

use super::save_file::traits::Savable;
use crate::prelude::RayScalar;
use crate::ray_tracer::save_file::SceneSaveError;
use cgmath::Vector3;
use rusqlite::Connection;
use std::f64::consts::FRAC_PI_4;
use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub struct Sun {
    pub phi: RayScalar,
    pub theta: RayScalar,
    /// radius in radians
    pub radius: RayScalar,
}
impl Sun {
    pub fn make_direction_vector(&self) -> Vector3<RayScalar> {
        let r = self.phi.cos();
        Vector3::new(r * self.theta.cos(), self.phi.sin(), r * self.theta.sin())
    }
}
impl Default for Sun {
    fn default() -> Self {
        Self {
            phi: FRAC_PI_4,
            theta: 0.,
            radius: 0.1,
        }
    }
}
impl Savable for Sun {
    fn database_name() -> &'static str {
        "sun"
    }

    fn make_schema(connection: &Connection) -> Result<(), SceneSaveError> {
        let sql = format!(
            "CREATE TABLE {name}({name}_id blob PRIMARY KEY, phi REAL NOT NULL, theta REAL NOT NULL, radius REAL NOT NULL) STRICT;",
            name = Self::database_name()
        );
        connection.execute(&sql, ())?;
        Ok(())
    }

    fn delete_schema(_connection: &mut Connection) {
        todo!()
    }

    fn save(&self, connection: &Connection) -> Result<Uuid, SceneSaveError> {
        let self_uuid = Uuid::new_v4();
        let sql_call = format!(
            "INSERT INTO {name}({name}_id, phi, theta, radius) VALUES (?1, ?2, ?3, ?4);",
            name = Self::database_name()
        );
        connection.execute(&sql_call, (self_uuid, self.phi, self.theta, self.radius))?;
        Ok(self_uuid)
    }

    fn load(_id: Uuid, _connection: &Connection) -> Result<Vec<Self>, SceneSaveError> {
        todo!()
    }
}

use super::{Background, Savable, SceneSaveError};

use crate::{
    prelude::{Ray, RayScalar, RgbColor},
    ray_tracer::sun::Sun,
};

use cgmath::{InnerSpace, Vector3};
use log::error;
use rusqlite::Connection;
use uuid::Uuid;
#[derive(Clone)]
pub struct SunSky {
    pub intensity: RayScalar,
    pub sun_radius: RayScalar,
    pub sun_theta: RayScalar,
    pub sun_phi: RayScalar,
    pub sun_brightness: RayScalar,
}
impl SunSky {
    pub fn new(sun: Sun, intensity: RayScalar, sun_brightness: RayScalar) -> Self {
        Self {
            intensity,
            sun_radius: sun.radius,
            sun_theta: sun.theta,
            sun_phi: sun.phi,
            sun_brightness,
        }
    }
}
impl Background for SunSky {
    fn color(&self, ray: Ray) -> RgbColor {
        let r = self.sun_phi.cos();

        let sun_ray = Vector3::new(
            r * self.sun_theta.cos(),
            self.sun_phi.sin(),
            r * self.sun_theta.sin(),
        );
        let sun_cos = sun_ray.dot(ray.direction.normalize());

        if sun_cos > self.sun_radius.cos() && sun_cos > 0.0 {
            self.sun_brightness * RgbColor::WHITE
        } else {
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
}
impl Default for SunSky {
    fn default() -> Self {
        Self::new(Sun::default(), 1.0, 1.0)
    }
}
impl Savable for SunSky {
    fn database_name() -> &'static str {
        "sun_sky"
    }

    fn make_schema(connection: &Connection) -> Result<(), SceneSaveError> {
        let name = <Self as Savable>::database_name();
        let sql_call = format!(
            "CREATE TABLE {}(\
                {}_id blob PRIMARY KEY,\
                intensity REAL NOT NULL, \
                sun_radius REAL NOT NULL, \
                sun_theta REAL NOT NULL, \
                sun_phi REAL NOT NULL, \
                sun_brightness REAL NOT NULL\
            ) STRICT;",
            name, name
        );
        connection.execute(&sql_call, ())?;
        Ok(())
    }

    fn delete_schema(_connection: &mut Connection) {
        todo!()
    }

    fn save(&self, connection: &Connection) -> Result<Uuid, SceneSaveError> {
        let self_uuid = Uuid::new_v4();
        let sql_call = format!(
            "INSERT INTO {name}({name}_id, intensity, sun_radius, sun_theta, sun_phi, sun_brightness) VALUES (?1, ?2, ?3, ?4, ?5, ?6);",
            name = Self::database_name()
        );
        connection.execute(
            &sql_call,
            (
                self_uuid,
                self.intensity,
                self.sun_radius,
                self.sun_theta,
                self.sun_phi,
                self.sun_brightness,
            ),
        )?;
        Ok(self_uuid)
    }
    fn load(id: Uuid, connection: &Connection) -> Result<Vec<Self>, SceneSaveError> {
        let statement = format!(
            "SELECT \
            intensity, sun_radius, sun_theta, sun_phi, sun_brightness \
            FROM {name} WHERE {name}_id = ?1
        ",
            name = Self::database_name()
        );
        let mut statement = connection.prepare(&statement)?;

        let query = statement.query_map([id], |row| {
            Ok(Self {
                intensity: row.get(0)?,
                sun_radius: row.get(1)?,
                sun_theta: row.get(2)?,
                sun_phi: row.get(3)?,
                sun_brightness: row.get(4)?,
            })
        })?;
        Ok(query
            .filter_map(|s| match s {
                Ok(s) => Some(s),
                Err(e) => {
                    error!("failed to load sun_sky reason: \"{:?}\"", e);
                    None
                }
            })
            .collect())
    }
}

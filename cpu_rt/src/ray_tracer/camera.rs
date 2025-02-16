use super::{
    ray_tracer_info::{Entity, EntityField},
    save_file::{traits::Savable, SceneSaveError},
};
use crate::prelude::*;

use cgmath::{num_traits::FloatConst, InnerSpace, Point3, Vector3};
use rusqlite::Connection;
use std::collections::HashMap;
use uuid::Uuid;

/// info used to construct camera
#[derive(Clone, Debug, PartialEq)]
pub struct CameraInfo {
    pub aspect_ratio: RayScalar,
    pub fov: RayScalar,
    pub origin: Point3<RayScalar>,
    pub look_at: Point3<RayScalar>,
    pub up_vector: Vector3<RayScalar>,
    pub aperture: RayScalar,
    pub focus_distance: RayScalar,
    pub start_time: RayScalar,
    pub end_time: RayScalar,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Camera {
    origin: Point3<RayScalar>,
    lower_left_corner: Point3<RayScalar>,
    horizontal: Vector3<RayScalar>,
    vertical: Vector3<RayScalar>,
    u: Vector3<RayScalar>,
    v: Vector3<RayScalar>,
    look_at: Point3<RayScalar>,
    lens_radius: RayScalar,
    start_time: RayScalar,
    end_time: RayScalar,
    up_vector: Vector3<RayScalar>,
    focus_distance: RayScalar,
    world_width: RayScalar,
    world_height: RayScalar,
    info: CameraInfo,
}
impl Camera {
    pub fn new(info: CameraInfo) -> Self {
        let theta = info.fov * RayScalar::PI() / 180.0;
        let h = (theta / 2.0).tan();
        let world_height = 2.0 * h;

        let world_width = info.aspect_ratio * world_height;

        let (w, u, v) = Self::calculate_w_u_v(info.origin, info.look_at, info.up_vector);
        let horizontal = info.focus_distance * world_width * u;

        let vertical = info.focus_distance * world_height * v;

        Self {
            origin: info.origin,
            horizontal,
            vertical,
            lower_left_corner: info.origin
                - horizontal / 2.0
                - vertical / 2.0
                - info.focus_distance * w,
            u,
            v,
            lens_radius: info.aperture / 2.0,
            start_time: info.start_time,
            end_time: info.end_time,
            look_at: info.look_at,
            up_vector: info.up_vector,
            focus_distance: info.focus_distance,
            world_width,
            world_height,
            info,
        }
    }
    fn calculate_w_u_v(
        origin: Point3<RayScalar>,
        look_at: Point3<RayScalar>,
        up_vector: Vector3<RayScalar>,
    ) -> (Vector3<RayScalar>, Vector3<RayScalar>, Vector3<RayScalar>) {
        let w = (origin - look_at).normalize();
        let u = up_vector.cross(w).normalize();
        let v = w.cross(u);
        (w, u, v)
    }
    pub fn get_ray(&self, u: RayScalar, v: RayScalar) -> Ray {
        let rd = self.lens_radius * Self::random_in_unit_disk();
        let offset = self.u * rd.x + self.v * rd.y;
        Ray {
            origin: self.origin,
            direction: self.lower_left_corner + u * self.horizontal + v * self.vertical
                - self.origin
                - offset,
            time: rand_scalar(self.start_time, self.end_time),
        }
    }
    fn random_in_unit_disk() -> Vector3<RayScalar> {
        loop {
            let p = Vector3::new(rand_scalar(-1.0, 1.0), rand_scalar(-1.0, 1.0), 0.0);
            if p.dot(p) < 1.0 {
                return p;
            }
        }
    }
    pub fn start_time(&self) -> RayScalar {
        self.start_time
    }
    pub fn end_time(&self) -> RayScalar {
        self.end_time
    }
    fn set_look_at(&mut self, look_at: Point3<RayScalar>) {
        let mut info = self.info.clone();
        info.look_at = look_at;
        *self = Self::new(info);
    }
    fn set_origin(&mut self, origin: Point3<RayScalar>) {
        let mut info = self.info.clone();
        info.origin = origin;
        *self = Self::new(info);
    }
}
impl Entity for Camera {
    fn name(&self) -> String {
        "camera".to_string()
    }
    fn fields(&self) -> HashMap<String, EntityField> {
        let mut map = HashMap::new();
        map.insert("origin".to_string(), EntityField::Point3(self.info.origin));
        map.insert(
            "look_at".to_string(),
            EntityField::Point3(self.info.look_at),
        );
        map
    }
    fn set_field(&mut self, key: String, value: EntityField) {
        match key.as_str() {
            "origin" => match value {
                EntityField::Point3(p) => self.set_origin(p),
                _ => panic!("invalid field type"),
            },
            "look_at" => match value {
                EntityField::Point3(p) => self.set_look_at(p),
                _ => panic!("invalid field type"),
            },
            _ => panic!("invalid field: {}", key),
        };
    }
}
impl Savable for Camera {
    fn database_name() -> &'static str {
        "camera"
    }

    fn make_schema(connection: &Connection) -> Result<(), SceneSaveError> {
        CameraInfo::make_schema(connection)?;
        let sql = format!(
            "CREATE TABLE {self_name}(\
                {self_name}_id BLOB PRIMARY KEY NOT NULL, \
                {camera_info_name}_id BLOB NOT NULL,\
                FOREIGN KEY({camera_info_name}_id) REFERENCES {camera_info_name}({camera_info_name}_id)\
            )STRICT;",
            self_name = <Self as Savable>::database_name(),
            camera_info_name = <CameraInfo as Savable>::database_name()
        );

        connection.execute(&sql, ())?;
        Ok(())
    }

    fn delete_schema(connection: &mut Connection) {
        todo!()
    }

    fn save(&self, connection: &Connection) -> Result<Uuid, SceneSaveError> {
        let info_uuid = self.info.save(connection)?;
        let self_uuid = Uuid::new_v4();

        let sql = format!(
            "INSERT INTO {self_name}({self_name}_id, {info_name}_id) VALUES (?1, ?2)",
            self_name = Self::database_name(),
            info_name = CameraInfo::database_name()
        );
        connection.execute(&sql, (self_uuid, info_uuid))?;
        Ok(self_uuid)
    }

    fn load(id: Uuid, connection: &Connection) -> Result<Vec<Self>, SceneSaveError> {
        let query = format!(
            "SELECT {camera_info_name}_id FROM {self_name} WHERE {self_name}_id = ?1",
            self_name = Self::database_name(),
            camera_info_name = CameraInfo::database_name()
        );
        let mut statement = connection.prepare(&query)?;
        let camera_uuids = statement.query_map([&id], |row| row.get::<_, Uuid>(0))?;
        Ok(camera_uuids
            .filter_map(|val| match val {
                Ok(v) => Some(v),
                Err(error) => {
                    error!("failed to read camera from database: {:?}", error);
                    None
                }
            })
            .filter_map(|info_id| match CameraInfo::load(info_id, connection) {
                Ok(info) => Some(info),
                Err(error) => {
                    error!("failed to read camera info from database: {:?}", error);
                    None
                }
            })
            .flatten()
            .map(|info| Camera::new(info))
            .collect::<Vec<_>>())
    }
}
impl Savable for CameraInfo {
    fn database_name() -> &'static str {
        "camera_info"
    }

    fn make_schema(connection: &Connection) -> Result<(), SceneSaveError> {
        let sql = format!(
            "CREATE TABLE {self_name}(\
                {self_name}_id BLOB PRIMARY KEY NOT NULL,\
                aspect_ratio REAL NOT NULL,\
                fov REAL NOT NULL,\
                origin_x REAL NOT NULL,\
                origin_y REAL NOT NULL,\
                origin_z REAL NOT NULL,\
                \
                look_at_x REAL NOT NULL,\
                look_at_y REAL NOT NULL,\
                look_at_z REAL NOT NULL,\
                \
                up_vector_x REAL NOT NULL,\
                up_vector_y REAL NOT NULL,\
                up_vector_z REAL NOT NULL,\
                \
                aperture REAL NOT NULL,\
                focus_distance REAL NOT NULL,\
                start_time REAL NOT NULL,\
                end_time REAL NOT NULL
            ) STRICT",
            self_name = <Self as Savable>::database_name()
        );
        connection.execute(&sql, ())?;
        Ok(())
    }

    fn delete_schema(connection: &mut Connection) {
        todo!()
    }

    fn save(&self, connection: &Connection) -> Result<Uuid, SceneSaveError> {
        let info_uuid = Uuid::new_v4();
        let sql = format!(
            "INSERT INTO {self_name}(\
                {self_name}_id,\
                aspect_ratio,\
                fov,\
                origin_x,\
                origin_y,\
                origin_z,\
                look_at_x,\
                look_at_y,\
                look_at_z,\
                up_vector_x,\
                up_vector_y,\
                up_vector_z,\
                aperture,\
                focus_distance,\
                start_time,\
                end_time
            ) VALUES (\
                ?1,\
                ?2,\
                ?3,\
                ?4,\
                ?5,\
                ?6,\
                ?7,\
                ?8,\
                ?9,\
                ?10,\
                ?11,\
                ?12,\
                ?13,\
                ?14,\
                ?15,\
                ?16\
            );",
            self_name = Self::database_name()
        );
        connection.execute(
            &sql,
            (
                info_uuid,
                self.aspect_ratio,
                self.fov,
                self.origin.x,
                self.origin.y,
                self.origin.z,
                self.look_at.x,
                self.look_at.y,
                self.look_at.z,
                self.up_vector.x,
                self.up_vector.y,
                self.up_vector.z,
                self.aperture,
                self.focus_distance,
                self.start_time,
                self.end_time,
            ),
        )?;
        Ok(info_uuid)
    }

    fn load(id: Uuid, connection: &Connection) -> Result<Vec<Self>, SceneSaveError> {
        let query = format!(
            "SELECT aspect_ratio, \
            fov, \
            origin_x, \
            origin_y, \
            origin_z, \
            look_at_x, \
            look_at_y, \
            look_at_z, \
            up_vector_x, \
            up_vector_y, \
            up_vector_z, \
            aperture, \
            focus_distance, \
            start_time, \
            end_time \
            FROM {self_name} WHERE \
            {self_name}_id = ?1",
            self_name = Self::database_name()
        );
        let mut statement = connection.prepare(&query)?;
        let statement_map = statement.query_map([id], |row| {
            Ok(Self {
                aspect_ratio: row.get(0)?,
                fov: row.get(1)?,
                origin: Point3::new(row.get(2)?, row.get(3)?, row.get(4)?),
                look_at: Point3::new(row.get(5)?, row.get(6)?, row.get(7)?),
                up_vector: Vector3::new(row.get(8)?, row.get(9)?, row.get(10)?),
                aperture: row.get(11)?,
                focus_distance: row.get(12)?,
                start_time: row.get(13)?,
                end_time: row.get(14)?,
            })
        })?;
        Ok(statement_map
            .filter_map(|v| match v {
                Ok(v) => Some(v),
                Err(err) => {
                    error!("failed to load camera info: \"{}\"", err);
                    None
                }
            })
            .collect::<Vec<_>>())
    }
}

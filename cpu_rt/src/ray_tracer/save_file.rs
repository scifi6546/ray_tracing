pub(crate) mod traits;
use super::{
    background::saver_loader as background_saver, camera::Camera, sun::Sun, world::WorldInfo,
    RayTracer,
};
use crate::prelude::RayScalar;
use log::error;
use log::info;
use rusqlite::{Connection, Error as SqliteError, OpenFlags};
use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use traits::Savable;
use uuid::{Error, Uuid};
#[derive(Debug)]
pub enum SceneSaveError {
    DatabaseError(SqliteError),
    FileSystemError(std::io::Error),
    SystemTimeError(std::time::SystemTimeError),
    UuidParseError(uuid::Error),
    NotFoundInDatabase(String),
}
impl From<SqliteError> for SceneSaveError {
    fn from(error: SqliteError) -> Self {
        Self::DatabaseError(error)
    }
}
impl From<std::io::Error> for SceneSaveError {
    fn from(error: std::io::Error) -> Self {
        Self::FileSystemError(error)
    }
}
impl From<std::time::SystemTimeError> for SceneSaveError {
    fn from(error: std::time::SystemTimeError) -> Self {
        Self::SystemTimeError(error)
    }
}
impl From<uuid::Error> for SceneSaveError {
    fn from(error: Error) -> Self {
        Self::UuidParseError(error)
    }
}
pub(crate) struct SceneFile {
    database_connection: Connection,
}
impl SceneFile {
    const CURRENT_VERSION: u32 = 0;
    /// creates new scene file from ray tracer
    fn new(save_path: PathBuf) -> Result<Self, SceneSaveError> {
        if save_path.exists() {
            std::fs::remove_file(&save_path)?;
        }
        let database_connection = Connection::open(save_path)?;

        Ok(Self {
            database_connection,
        })
    }
    fn save(&self, ray_tracer: &RayTracer) -> Result<(), SceneSaveError> {
        self.database_connection.execute(
            "create TABLE metadata(version INTEGER, save_time INTEGER);",
            (),
        )?;
        let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        self.database_connection.execute(
            "INSERT INTO metadata(version, save_time) VALUES (?1, ?2)",
            (Self::CURRENT_VERSION, time),
        )?;
        background_saver::make_schema(&self.database_connection)?;
        Camera::make_schema(&self.database_connection)?;
        Sun::make_schema(&self.database_connection)?;
        let scene_table_sql = format!(
            "CREATE TABLE scene (\
            scene_id BLOB PRIMARY KEY,\
            shader TEXT NOT NULL,\
            background_id BLOB NOT NULL,\
            camera_id BLOB NOT NULL,\
            {sun}_id BLOB,
            FOREIGN KEY(background_id) REFERENCES background(background_id)\
            FOREIGN KEY({camera}_id) REFERENCES {camera}({camera}_id)\
            FOREIGN KEY({sun}_id) REFERENCES {sun}({sun}_id)\
           ) STRICT;",
            camera = Camera::database_name(),
            sun = Sun::database_name()
        );
        self.database_connection.execute(&scene_table_sql, ())?;

        let background_id = background_saver::save_background(
            ray_tracer.world.background.as_ref(),
            &self.database_connection,
        )?;
        let camera_id = ray_tracer.world.camera.save(&self.database_connection)?;
        let scene_id = Uuid::new_v4();
        let sun_id = if let Some(sun) = ray_tracer.world.sun {
            Some(sun.save(&self.database_connection)?)
        } else {
            None
        };
        self.database_connection.execute(
            "INSERT INTO scene (scene_id, shader, background_id, camera_id, sun_id) VALUES (?1, ?2, ?3, ?4, ?5);",
            (scene_id, "todo", background_id, camera_id, sun_id),
        )?;

        error!("todo save entities and stuff");
        Ok(())
    }
    fn load(path: PathBuf) -> Result<WorldInfo, SceneSaveError> {
        let connection = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        let mut statement = connection.prepare("SELECT scene_id, camera_id FROM scene")?;
        let (scene_id, camera_id): (Uuid, Uuid) = statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .next()
            .unwrap()?;

        info!("todo create objects and lights");

        info!("todo load sun");
        let background = background_saver::load_background(scene_id, &connection)?;
        let camera = Camera::load_one(camera_id, &connection)?;
        Ok(WorldInfo {
            objects: vec![],
            lights: vec![],
            background,
            camera,
            sun: None,
        })
    }
    /// creates a new scene file builder
    pub(crate) fn builder(save_path: PathBuf) -> SceneFileBuilder {
        SceneFileBuilder { save_path }
    }
    pub(crate) const FILE_EXTENSION: &'static str = "mscene";
}

pub(crate) struct SceneFileBuilder {
    save_path: PathBuf,
}
impl SceneFileBuilder {
    pub(crate) fn save(self, ray_tracer: &RayTracer) -> Result<(), SceneSaveError> {
        let file = SceneFile::new(self.save_path)?;
        file.save(ray_tracer)
    }
    pub(crate) fn load(self) -> Result<WorldInfo, SceneSaveError> {
        SceneFile::load(self.save_path)
    }
}

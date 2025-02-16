use super::SceneSaveError;
use rusqlite::Connection;
use uuid::Uuid;
/// implements interface for saving scenes as sqlite databases. Each entity type will have its own table
/// however each entity must share the same table with entities of the same type
pub(crate) trait DynSavable {
    /// name of table that will be used for object
    fn database_name(&self) -> String;
    /// creates schema for table
    fn make_schema(&self, connection: &Connection) -> Result<(), SceneSaveError>;
    /// drops schema
    fn delete_schema(&self, connection: &mut Connection);
    /// saves to database and returns id of object
    fn save(&self, connection: &Connection) -> Result<Uuid, SceneSaveError>;
}
pub(crate) trait Savable: Sized {
    fn database_name() -> &'static str;
    fn make_schema(connection: &Connection) -> Result<(), SceneSaveError>;
    fn delete_schema(connection: &mut Connection);
    fn save(&self, connection: &Connection) -> Result<Uuid, SceneSaveError>;
    fn load_dyn(id: Uuid, connection: &Connection) -> Result<Vec<Box<Self>>, SceneSaveError> {
        Ok(Self::load(id, connection)?
            .drain(..)
            .map(|item| Box::new(item))
            .collect())
    }
    fn load(id: Uuid, connection: &Connection) -> Result<Vec<Self>, SceneSaveError>;
    /// loads first instance of self found in database
    fn load_one(id: Uuid, connection: &Connection) -> Result<Self, SceneSaveError> {
        let mut items = Self::load(id, connection)?;
        if let Some(item) = items.pop() {
            Ok(item)
        } else {
            Err(SceneSaveError::NotFoundInDatabase(
                <Self as Savable>::database_name().to_string(),
            ))
        }
    }
}
impl<T: Savable> DynSavable for T {
    fn database_name(&self) -> String {
        Self::database_name().to_string()
    }
    fn make_schema(&self, connection: &Connection) -> Result<(), SceneSaveError> {
        Self::make_schema(connection)
    }
    fn delete_schema(&self, connection: &mut Connection) {
        Self::delete_schema(connection)
    }
    fn save(&self, connection: &Connection) -> Result<Uuid, SceneSaveError> {
        Savable::save(self, connection)
    }
}

use super::{
    super::save_file::traits::Savable, Background, ConstantColor, SceneSaveError, Sky, SunSky,
};

use log::info;
use rusqlite::Connection;
use std::collections::HashMap;
use uuid::Uuid;
type BackgroundCtor =
    fn(id: Uuid, &Connection) -> Result<Vec<Box<dyn Background + 'static + Send>>, SceneSaveError>;
fn get_names_loader_map() -> HashMap<String, BackgroundCtor> {
    fn background_ctor_adaptor<T: Background + 'static + Savable + Send>(
        id: Uuid,
        conn: &Connection,
    ) -> Result<Vec<Box<dyn Background + 'static + Send>>, SceneSaveError> {
        Ok(<T as Savable>::load_dyn(id, conn)?
            .drain(..)
            .map(|v| v as Box<dyn Background + 'static + Send>)
            .collect())
    }
    fn insert<T: Background + 'static + Savable>(map: &mut HashMap<String, BackgroundCtor>) {
        map.insert(
            <T as Savable>::database_name().to_string(),
            background_ctor_adaptor::<T> as BackgroundCtor,
        );
    }

    let mut map = HashMap::new();

    insert::<ConstantColor>(&mut map);
    insert::<Sky>(&mut map);
    insert::<SunSky>(&mut map);
    map
}
fn get_all_names() -> Vec<&'static str> {
    vec![
        <ConstantColor as Savable>::database_name(),
        <Sky as Savable>::database_name(),
        <SunSky as Savable>::database_name(),
    ]
}

fn make_schemas(connection: &Connection) -> Result<(), SceneSaveError> {
    ConstantColor::make_schema(connection)?;
    Sky::make_schema(connection)?;
    SunSky::make_schema(connection)?;
    Ok(())
}
pub(crate) fn make_schema(connection: &Connection) -> Result<(), SceneSaveError> {
    make_schemas(connection)?;
    let database_names = get_all_names();
    let type_names = database_names
        .iter()
        .map(|name| format!("'{}_id' blob", name))
        .fold(String::new(), |acc, x| acc + " ,\n" + &x);
    let foreign_keys = database_names
        .iter()
        .map(|name| format!("FOREIGN KEY({}_id) REFERENCES {}({}_id)", name, name, name))
        .fold(
            String::new(),
            |acc, x| if acc != "" { acc + ", " + &x } else { x },
        );
    let full_statement = format!(
        "CREATE TABLE background (background_id blob{},PRIMARY KEY(background_id), {});",
        type_names, foreign_keys
    );
    info!("{}", full_statement);
    connection.execute(&full_statement, ())?;
    Ok(())
}
pub(crate) fn save_background(
    background: &dyn Background,
    connection: &Connection,
) -> Result<Uuid, SceneSaveError> {
    let background_ty_id = background.save(connection)?;
    let background_id = Uuid::new_v4();
    let statement = format!(
        "INSERT INTO background(background_id, {}_id) VALUES (?1,?2);",
        background.database_name()
    );
    connection.execute(&statement, (background_id, background_ty_id))?;
    Ok(background_id)
}
pub(crate) fn load_background(
    scene_id: Uuid,
    connection: &Connection,
) -> Result<Box<dyn Background + Send>, SceneSaveError> {
    let background_id: Uuid = {
        let mut background_statement =
            connection.prepare("SELECT background_id FROM scene WHERE scene_id = ?1")?;
        let mut background_query = background_statement.query([scene_id])?;
        let mut row = background_query.next()?.unwrap();

        row.get(0)?
    };

    let (background_name, background_uuid) = {
        let names_id_pair = get_all_names()
            .drain(..)
            .map(|name| (name.to_string(), format!("{}_id", name)))
            .collect::<Vec<_>>();
        let names = names_id_pair
            .iter()
            .fold(String::new(), |acc, (_name, id)| {
                if acc != "" {
                    acc + ", " + &id
                } else {
                    id.clone()
                }
            });
        let query_string = format!("SELECT {} FROM background WHERE background_id=?1;", names);
        let mut statement = connection.prepare(&query_string)?;
        let mut query = statement.query([background_id])?;
        let row = query.next()?.unwrap();
        names_id_pair
            .iter()
            .enumerate()
            .filter_map(|(index, (name, column_name))| {
                if let Some(uuid) = row
                    .get::<_, Option<Uuid>>(index)
                    .expect("failed to get column")
                {
                    Some((name.to_string(), uuid))
                } else {
                    None
                }
            })
            .next()
            .expect("No background found")
    };
    let constructor_map = get_names_loader_map();
    let constructor = constructor_map.get(&background_name).unwrap();
    let mut backgrounds = constructor(background_uuid, connection)?;
    Ok(backgrounds.pop().unwrap())
}

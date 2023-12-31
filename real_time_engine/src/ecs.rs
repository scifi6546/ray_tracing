mod render;

use bevy::prelude::*;
pub fn ecs_stuff() {
    App::new()
        .add_plugin(render::RenderPlugin)
        .add_startup_system(add_ships)
        .add_system(get_ships)
        .run();
}
fn hello_system() {
    println!("hi")
}
#[derive(Component, Debug)]
struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
fn add_ships(mut commands: Commands) {
    commands.spawn((Position::new(1, 1, 1)));
}
fn get_ships(query: Query<&Position, ()>) {
    for pos in query.iter() {
        println!("pos: {}", pos)
    }
}
impl Position {
    pub fn zero() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}
impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{},{},{}]", self.x, self.y, self.z)
    }
}

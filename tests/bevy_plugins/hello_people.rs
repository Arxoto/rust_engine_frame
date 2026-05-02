use bevy::prelude::*;

#[derive(Resource)]
pub struct Chats(pub Vec<String>);

#[derive(Component)]
pub struct Person;

#[derive(Component)]
pub struct Name(pub String);

/// 添加实体
fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("Elaina Proctor".to_string())));
    commands.spawn((Person, Name("Renzo Hume".to_string())));
    commands.spawn((Person, Name("Zayna Nieves".to_string())));
}

/// 对实体进行查询并输出
fn greet_people(query: Query<&Name, With<Person>>, mut chats: ResMut<Chats>) {
    for name in &query {
        chats.0.push(format!("Hello, {}!", name.0));
    }
}

pub struct HelloPeoplePlugin;

impl Plugin for HelloPeoplePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Chats(vec![]))
            .add_systems(Startup, add_people)
            .add_systems(Update, greet_people);
    }
}

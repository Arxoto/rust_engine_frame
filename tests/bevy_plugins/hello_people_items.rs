use bevy::prelude::*;

#[derive(Resource)]
pub struct Chats(pub Vec<String>);

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

#[derive(Component)]
struct Item(String);

#[derive(Component)]
struct Backpack(Vec<Entity>);

/// 添加实体
fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("qqq".to_string()), Backpack(vec![])));
    commands.spawn((Person, Name("www".to_string()), Backpack(vec![])));
    commands.spawn((Person, Name("eee".to_string()), Backpack(vec![])));
}

fn add_items(mut commands: Commands, query: Query<(&Name, &mut Backpack), With<Person>>) {
    for (name, mut backpack) in query {
        let items = match name.0.as_str() {
            "qqq" => vec!["Sword", "Shield"],
            "www" => vec!["Potion", "Scroll"],
            "eee" => vec!["Bow", "Arrow"],
            _ => vec![],
        };
        for item_name in items {
            let item_entity = commands.spawn(Item(item_name.to_string())).id();
            backpack.0.push(item_entity);
        }
    }
}

fn show_backpacks(
    query: Query<(&Name, &Backpack), With<Person>>,
    item_query: Query<&Item>,
    mut chats: ResMut<Chats>,
) {
    for (name, backpack) in query {
        chats.0.push(format!("{}'s backpack contains:", name.0));
        for &item_entity in &backpack.0 {
            if let Ok(item) = item_query.get(item_entity) {
                chats.0.push(format!("- {}", item.0));
            }
        }
    }
}

pub struct HelloPeopleItemsPlugin;

impl Plugin for HelloPeopleItemsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Chats(vec![]))
            .add_systems(Startup, (add_people, add_items).chain())
            .add_systems(Update, show_backpacks);
    }
}

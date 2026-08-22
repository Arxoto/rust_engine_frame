#![cfg(feature = "bevyproj")]

pub mod bevy_plugins;

use bevy::prelude::*;

#[test]
fn test_people() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy_plugins::hello_people::HelloPeoplePlugin);

    use bevy_plugins::hello_people::{Chats, Name, Person};

    app.update();
    let mut people_count = 0;
    for (_person, name) in app
        .world_mut()
        .query::<(&Person, &Name)>()
        .iter(app.world())
    {
        match name.0.as_str() {
            "Elaina Proctor" => {
                people_count += 1;
            }
            "Renzo Hume" => {
                people_count += 1;
            }
            "Zayna Nieves" => {
                people_count += 1;
            }
            _ => {
                panic!("no!!!")
            }
        }
    }
    assert_eq!(people_count, 3, "There should be 3 people in the world");

    let chats = app.world().resource::<Chats>();
    assert_eq!(
        chats.0,
        vec![
            "Hello, Elaina Proctor!".to_string(),
            "Hello, Renzo Hume!".to_string(),
            "Hello, Zayna Nieves!".to_string()
        ],
        "The chats should contain greetings for all people"
    );
}

#[test]
fn test_people_items() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy_plugins::hello_people_items::HelloPeopleItemsPlugin);

    use bevy_plugins::hello_people_items::Chats;

    app.update();

    let chats = app.world().resource::<Chats>();
    assert_eq!(
        chats.0,
        vec![
            "qqq's backpack contains:".to_string(),
            "- Sword".to_string(),
            "- Shield".to_string(),
            "www's backpack contains:".to_string(),
            "- Potion".to_string(),
            "- Scroll".to_string(),
            "eee's backpack contains:".to_string(),
            "- Bow".to_string(),
            "- Arrow".to_string(),
        ],
        "The chats should contain the items in each person's backpack"
    );
}

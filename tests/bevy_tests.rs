#![cfg(feature = "bevyproj")]

pub mod bevy_plugins;

use bevy::prelude::*;
use rust_engine_frame::base_lib::{
    cores::{
        timers::{
            tick_timer::TickTimer,
            tiny_timer::{HasTimer, Tickable},
        },
        unify_types::time_type,
    },
    eff_attr_prop::{
        attr_eff::{AttrEffect, AttrEffectType},
        attr_systems::{clean_expired_element, try_refresh_dirty_attr},
        attrs::Attr,
        effects::Effect,
        upsert_container::UpsertContainer,
    },
};

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

/// 本地 newtype:把 base_lib 的属性效果容器包装为 Bevy Resource(孤儿规则适配)
#[derive(Resource)]
struct EffsResource(UpsertContainer<AttrEffect<String, TickTimer>>);

/// 本地 newtype:属性值
#[derive(Resource)]
struct AttrResource(Attr);

/// 每帧驱动 base_lib 的真实逻辑:推进效果计时器 → 清理过期 → 刷新脏属性
fn per_entity_tick_system(mut attr: ResMut<AttrResource>, mut effs: ResMut<EffsResource>) {
    // 固定帧时长,保证测试确定性(不依赖 Bevy 真实时钟)
    let delta: time_type::T = time_type::unit::<1>();
    for eff in effs.0.iter_mut() {
        eff.get_timer_mut().tick(delta);
    }
    clean_expired_element(&mut effs.0, ());
    try_refresh_dirty_attr(&mut attr.0, &mut effs.0);
}

/// 验证"无引擎依赖"承诺:同一套 base_lib 属性刷新链在 Bevy System 下可运行
#[test]
fn base_lib_attr_chain_runs_under_bevy() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, per_entity_tick_system);

    // 一个 2 帧后过期的效果(基础加法 +20)和一个无限效果(基础加法 +5)
    let mut effs = UpsertContainer::<AttrEffect<String, TickTimer>>::default();
    effs.upsert_ele(
        AttrEffect::new(
            AttrEffectType::BasicAdd,
            Effect::<String>::new_form("buff", "short", 20.0),
            TickTimer::new(time_type::unit::<2>()),
        ),
        |_, _| {},
    );
    effs.upsert_ele(
        AttrEffect::new(
            AttrEffectType::BasicAdd,
            Effect::<String>::new_form("buff", "inf", 5.0),
            TickTimer::inf(),
        ),
        |_, _| {},
    );
    app.insert_resource(AttrResource(Attr::new(100.0)));
    app.insert_resource(EffsResource(effs));

    // 帧 1:short 未过期 → 100 + 20 + 5
    app.update();
    assert_eq!(
        app.world().resource::<AttrResource>().0.get_current(),
        125.0
    );

    // 帧 2:short 过期被清理 → 100 + 5
    app.update();
    assert_eq!(
        app.world().resource::<AttrResource>().0.get_current(),
        105.0
    );

    // 帧 3:无变更,脏标记未置位,属性保持
    app.update();
    assert_eq!(
        app.world().resource::<AttrResource>().0.get_current(),
        105.0
    );
}

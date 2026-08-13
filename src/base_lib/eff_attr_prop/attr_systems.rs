use crate::base_lib::{
    cores::{
        design_patterns::UnitedWith,
        timers::{
            static_timer::{StaticTimeline, StaticTimer},
            tick_timer::TickTimer,
            tiny_timer::{HasTimer, Tickable, TimerView},
        },
        unify_types::FixedName,
    },
    eff_attr_prop::{
        attr_eff::AttrEffect,
        attrs::Attr,
        upsert_container::{Upsert, UpsertContainer, UpsertContainerCleaner},
    },
};

/// todo 改造成更符合 ECS 标准的 System ，各个功能函数分开，老化效果和更新属性的函数也拆开
pub fn process_tick<S: FixedName>(
    delta: f64,
    timeline: &mut StaticTimeline,
    cleaner: &mut UpsertContainerCleaner,
    attr_effs: &mut [(&mut Attr, &mut UpsertContainer<AttrEffect<S, StaticTimer>>)],
) {
    timeline.0.tick(delta);

    // 老化效果 刷新属性值
    for (attr, effs) in &mut *attr_effs {
        try_update_attr(attr, effs, timeline);
    }

    // ===========================
    // 规整处理，业务无关
    // ===========================

    // 清理容器空洞
    let should_clean_period = UpsertContainerCleaner::get_default_period();
    let should_clean_hole = cleaner.should_clean_hole(delta, should_clean_period);
    if should_clean_hole {
        for (_, effs) in &mut *attr_effs {
            cleaner.do_clean_hole(effs);
        }
    }

    // 重置时间线
    let mut should_restart_timeline = true;
    for (_, effs) in &mut *attr_effs {
        should_restart_timeline &= effs.ele_empty();
    }
    if should_restart_timeline {
        timeline.reset_timeline();
    }
}

/// 老化效果 刷新属性值
fn try_update_attr<S: FixedName>(
    attr: &mut Attr,
    effs: &mut UpsertContainer<AttrEffect<S, StaticTimer>>,
    timeline: &StaticTimeline,
) {
    effs.delete_ele(|eff| eff.get_timer().of_timer(timeline).is_completed());

    if effs.is_changed() {
        effs.reset_changed_flag();

        attr.refresh_value(effs.iter_ele());
    }
}

/// 老化过期元素
///
/// 已知存在问题，也是 Rust 传统类型求解器 (Old Trait Solver) 中最经典的痛点之一
/// - 高阶生命周期 (HRTB) 与关联类型规范化 (Projection Normalization) 的耦合死锁
/// - 旧求解器缺乏“延迟承诺”能力。当遇到高阶生命周期 for<'b> 时，它必须“提前”把关联类型 <...>::Target 展开
/// - 但此时类型推断变量（如 With ）还没确定，导致它去盲目匹配 Blanket Implementation 时触发了歧义，进而编译报错
pub fn clean_expired_element<E, With>(ll: &mut UpsertContainer<E>, with: With)
where
    E: Upsert + HasTimer,
    With: Copy,
    for<'b> &'b <E as HasTimer>::Timer: UnitedWith<With>,
    for<'b> <&'b <E as HasTimer>::Timer as UnitedWith<With>>::IntoTarget: TimerView,
{
    ll.delete_ele(|ele| {
        let timer = ele.get_timer();
        let united_timer = timer.unite_into(with);
        united_timer.is_completed()
    });
}

pub fn test() {
    let mut ll = UpsertContainer::<AttrEffect<String, StaticTimer>>::default();
    clean_expired_element::<_, &StaticTimeline>(&mut ll, &StaticTimeline::new());

    let mut ll = UpsertContainer::<AttrEffect<String, TickTimer>>::default();
    clean_expired_element::<AttrEffect<String, TickTimer>, ()>(&mut ll, ());
}

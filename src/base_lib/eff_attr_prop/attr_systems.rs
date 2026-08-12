use crate::base_lib::{
    cores::{
        design_patterns::{Union, UnitedInto},
        timers::{
            static_timer::{StaticTimeline, StaticTimer},
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
/// todo test
///
/// 注意，这里约束 &E 让其返回的是所有权，需要测试 E 直接拥有 TinyTickTimer 的情况
/// 其返回引用应该没有实现 TimerView 会导致报错
pub fn clean_expired_element<'a, E, Timer, Ctx, Target>(
    ll: &'a mut UpsertContainer<E>,
    ctx: &'a Ctx,
) where
    E: Upsert + HasTimer<Timer = Timer>,
    for<'b> &'b Timer: UnitedInto<&'a Ctx, Target>,
    Target: TimerView,
{
    ll.delete_ele(|ele| ele.get_timer().unite_into(ctx).is_completed());
}

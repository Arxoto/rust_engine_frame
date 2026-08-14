use crate::base_lib::{
    cores::{
        timers::{
            static_timer::{StaticTimeline, StaticTimer},
            tick_timer::TickTimer,
            tiny_timer::{HasTimer, Tickable, TimerView},
        },
        unify_types::{FixedName, time_type},
    },
    eff_attr_prop::{
        attr_eff::AttrEffect,
        attrs::Attr,
        upsert_container::{Upsert, UpsertContainer, UpsertContainerCleaner},
    },
};

/// todo 改造成更符合 ECS 标准的 System ，各个功能函数分开，老化效果和更新属性的函数也拆开
pub fn process_tick<S: FixedName>(
    delta: time_type::T,
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
    let should_clean_period = time_type::DEFAULT_REFRESH_PERIOD;
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
    effs.delete_ele(|eff| eff.get_timer().is_completed(timeline));

    if effs.is_changed() {
        effs.reset_changed_flag();

        attr.refresh_value(effs.iter_ele());
    }
}

/// 老化过期元素
pub fn clean_expired_element<'a, E, Ctx>(ll: &mut UpsertContainer<E>, ctx: Ctx)
where
    Ctx: Copy,
    E: Upsert + HasTimer,
    <E as HasTimer>::Timer: TimerView<Ctx<'a> = Ctx>,
{
    ll.delete_ele(|ele| ele.get_timer().is_completed(ctx));
}

pub fn test() {
    let mut ll = UpsertContainer::<AttrEffect<String, StaticTimer>>::default();
    clean_expired_element(&mut ll, &StaticTimeline::new());

    let mut ll = UpsertContainer::<AttrEffect<String, TickTimer>>::default();
    clean_expired_element(&mut ll, ());
}

use crate::base_lib::{
    cores::{
        design_patterns::WithContext,
        static_timer::{StaticTimeline, StaticTimer},
        tiny_timer::{FlowingTimerReadonly, TickTimer},
        unify_types::FixedName,
    },
    eff_attr_prop::{
        attr_eff::AttrEffect,
        attrs::Attr,
        upsert_container::{UpsertContainer, UpsertContainerCleaner},
    },
};

/// todo
/// 这样的实现本身是一个很面向对象的写法
/// 确认是否要更 ECS 一点，把各个不同的处理逻辑拆开
/// 还有看一下是否要把 attr 和 effs 捆绑在一块，【确认每次修改 effs 是否都必定修改 attr】
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
        timeline.restart_timeline();
    }
}

/// 老化效果 刷新属性值
fn try_update_attr<S: FixedName>(
    attr: &mut Attr,
    effs: &mut UpsertContainer<AttrEffect<S, StaticTimer>>,
    timeline: &StaticTimeline,
) {
    effs.delete_ele(|eff| {
        let timer = eff.get_timer();
        timer.with_ctx(timeline).is_finished()
    });

    if effs.is_changed() {
        effs.reset_changed_flag();

        attr.refresh_value(effs.iter_ele());
    }
}

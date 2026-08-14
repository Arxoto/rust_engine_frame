use crate::base_lib::{
    cores::{
        timers::{
            static_timer::{StaticTimeline, StaticTimer},
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

/// 适用于面向对象去调用
///
/// - ECS 的 System 模式直接使用内部的函数
pub fn process_tick<S: FixedName>(
    delta: time_type::T,
    timeline: &mut StaticTimeline,
    cleaner: &mut UpsertContainerCleaner,
    attr_effs: &mut [(&mut Attr, &mut UpsertContainer<AttrEffect<S, StaticTimer>>)],
) {
    timeline.0.tick(delta);

    for (attr, effs) in &mut *attr_effs {
        clean_expired_element(effs, timeline);
        try_refresh_dirty_attr(attr, effs);
    }

    // 【规整处理，业务无关】

    // 惰性迭代器
    let ll = attr_effs.iter_mut().map(|(_, effs)| &mut **effs);
    try_clean_hole(delta, ll, cleaner);

    // 基本不会需要重置时间线，注释掉
    // let timers_iter = attr_effs
    //     .iter_mut()
    //     .map(|(_, effs)| &mut **effs)
    //     .flat_map(|effs| effs.iter_mut())
    //     .map(|eff| eff.get_timer_mut());
    // try_reset_timeline(timeline, timers_iter);
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

/// 刷新脏属性
pub fn try_refresh_dirty_attr<S: FixedName, Timer>(
    attr: &mut Attr,
    effs: &mut UpsertContainer<AttrEffect<S, Timer>>,
) {
    if effs.is_changed() {
        effs.reset_changed_flag();

        attr.refresh_value(effs.iter_ele());
    }
}

/// 清理容器空洞【规整处理，业务无关】
///
/// ```
/// # use rust_engine_frame::base_lib::cores::timers::static_timer::StaticTimer;
/// # use rust_engine_frame::base_lib::cores::unify_types::time_type;
/// # use rust_engine_frame::base_lib::eff_attr_prop::attr_eff::AttrEffect;
/// # use rust_engine_frame::base_lib::eff_attr_prop::attr_systems::try_clean_hole;
/// # use rust_engine_frame::base_lib::eff_attr_prop::attrs::Attr;
/// # use rust_engine_frame::base_lib::eff_attr_prop::upsert_container::UpsertContainer;
/// # use rust_engine_frame::base_lib::eff_attr_prop::upsert_container::UpsertContainerCleaner;
/// #
/// # let delta: time_type::T = time_type::ZERO;
/// # let mut cleaner: UpsertContainerCleaner = UpsertContainerCleaner::default();
/// type Effs = UpsertContainer<AttrEffect<String, StaticTimer>>;
/// let attr_effs: &mut [(&mut Attr, &mut Effs)] = &mut [];
///
/// let effs = attr_effs.iter_mut().map(|(_, effs)| &mut **effs); // 无法 Cpoy `&mut` 手动解引用 `&mut *`
/// try_clean_hole(delta, effs, &mut cleaner);
/// ```
pub fn try_clean_hole<'a, E: Upsert + 'a>(
    delta: time_type::T,
    ll: impl Iterator<Item = &'a mut UpsertContainer<E>>,
    cleaner: &mut UpsertContainerCleaner,
) {
    let should_clean_period = time_type::DEFAULT_REFRESH_PERIOD;
    let should_clean_hole = cleaner.should_clean_hole(delta, should_clean_period);
    if should_clean_hole {
        for effs in ll {
            cleaner.do_clean_hole(effs);
        }
    }
}

/// 重置时间线（使用 f64 或 Duration 作为时间类型，基本无需重置时间线）
pub fn try_reset_timeline<'a>(
    timeline: &mut StaticTimeline,
    timers_iter: impl Iterator<Item = &'a mut StaticTimer>,
) {
    let should_reset_timeline = timeline.current_time() >= time_type::RESET_TIMELINE_PERIOD;
    if should_reset_timeline {
        let diff = timeline.reset_timeline_and_get_diff();
        for ele in timers_iter {
            ele.fix_timeline_diff(diff);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::base_lib::cores::timers::tick_timer::TickTimer;

    use super::*;

    #[test]
    fn test_clean_expired_element() {
        let mut attr = Attr::default();

        let mut effs = UpsertContainer::<AttrEffect<String, StaticTimer>>::default();
        clean_expired_element(&mut effs, &StaticTimeline::new());
        try_refresh_dirty_attr(&mut attr, &mut effs);

        let mut effs = UpsertContainer::<AttrEffect<String, TickTimer>>::default();
        clean_expired_element(&mut effs, ());
        try_refresh_dirty_attr(&mut attr, &mut effs);
    }
}

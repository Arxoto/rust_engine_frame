//! 属性相关的 System 层
//!
//! ## 属性计算管线
//!
//! 首先刷新 [`super::stat_attrs`] [`super::bound_attrs`] 他们一般作为计算公式里的源端
//!
//! 而后刷新 [`super::bounded_attrs`] 应用计算公式得到结果
//!
//! 其中，有界属性的效果可分为两类：“无论如何都生效”和“根据结果决定是否生效”，他们的计算顺序应有区分
//!
//! - “无论如何都生效”的效果，应该先计算
//!   - 若具有层级，则使用复合属性 [`super::attr_layers`] ，先聚合同类效果再生效计算，以减少多层属性的计算次数
//!   - 每层计算完成后应该进行钳制，以确定下一层应生效多少效果值
//! - “根据结果决定是否生效”的效果，在后面计算，并且应该要求他们的顺序是确定的
//!   - 每次预判断结果时，都应结合上下限考虑，并且生效后应用钳制
//! - 最后提交有界属性的本次修改，作为下一帧快照值

use crate::base_lib::{
    cores::{
        timers::{
            static_timer::{StaticTimeline, StaticTimer},
            tiny_timer::{HasTimer, TimerView},
        },
        unify_types::{FixedName, time_type},
    },
    eff_attr::{
        bound_attr_effs::BoundAttrEff,
        bound_attrs::BoundAttr,
        bounded_attrs::BoundedAttr,
        stat_attr_effs::StatAttrEff,
        stat_attrs::StatAttr,
        upsert_container::{Upsert, UpsertContainer, UpsertContainerCleaner},
    },
};

/// 老化过期元素
pub fn clean_expired_element<'a, E, Ctx>(ll: &mut UpsertContainer<E>, ctx: Ctx)
where
    Ctx: Copy,
    E: Upsert + HasTimer,
    <E as HasTimer>::Timer: TimerView<Ctx<'a> = Ctx>,
{
    ll.delete_ele(|ele| ele.get_timer().is_completed(ctx));
}

/// 刷新 [`StatAttr`] 脏属性，应在帧开头触发
pub fn try_refresh_dirty_stat_attr<S: FixedName, Timer>(
    attr: &mut StatAttr,
    effs: &mut UpsertContainer<StatAttrEff<S, Timer>>,
) {
    if effs.is_changed() {
        effs.reset_changed_flag();

        attr.refresh_value(effs.iter_ele());
    }
}

/// 刷新 [`BoundAttr`] 脏属性，应在帧开头触发
pub fn try_refresh_dirty_bound_attr<S: FixedName, Timer>(
    attr: &mut BoundAttr,
    effs: &mut UpsertContainer<BoundAttrEff<S, Timer>>,
) {
    if effs.is_changed() {
        effs.reset_changed_flag();

        attr.refresh_value(effs.iter_ele());
    }
}

/// 提交有界属性的修改
#[inline]
pub fn do_commit_bounded_attr(attr: &mut BoundedAttr) {
    attr.commit_pending_value();
}

/// 清理容器空洞【规整处理，业务无关】(薄委托于 [`UpsertContainerCleaner::clean_holes`])
///
/// ```
/// # use rust_engine_frame::base_lib::cores::timers::static_timer::StaticTimer;
/// # use rust_engine_frame::base_lib::cores::unify_types::time_type;
/// # use rust_engine_frame::base_lib::eff_attr::stat_attr_effs::StatAttrEff;
/// # use rust_engine_frame::base_lib::eff_attr::stat_attrs::StatAttr;
/// # use rust_engine_frame::base_lib::eff_attr::attr_systems::try_clean_hole;
/// # use rust_engine_frame::base_lib::eff_attr::upsert_container::UpsertContainer;
/// # use rust_engine_frame::base_lib::eff_attr::upsert_container::UpsertContainerCleaner;
/// #
/// # let delta: time_type::T = time_type::ZERO;
/// # let mut cleaner: UpsertContainerCleaner = UpsertContainerCleaner::default();
/// type Effs = UpsertContainer<StatAttrEff<String, StaticTimer>>;
/// let attr_effs: &mut [(&mut StatAttr, &mut Effs)] = &mut [];
///
/// let effs = attr_effs.iter_mut().map(|(_, effs)| &mut **effs); // 无法 Cpoy `&mut` 手动解引用 `&mut *`
/// try_clean_hole(delta, effs, &mut cleaner);
/// ```
pub fn try_clean_hole<'a, E: Upsert + 'a>(
    delta: time_type::T,
    ll: impl Iterator<Item = &'a mut UpsertContainer<E>>,
    cleaner: &mut UpsertContainerCleaner,
) {
    cleaner.clean_holes(delta, ll);
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
    use crate::base_lib::cores::timers::{tick_timer::TickTimer, tiny_timer::Tickable};

    use super::*;

    /// 同时支持 [`StaticTimer`] [`TickTimer`]
    #[test]
    fn test_clean_expired_element() {
        let mut attr = StatAttr::new(0.0);

        let mut effs = UpsertContainer::<StatAttrEff<String, StaticTimer>>::default();
        clean_expired_element(&mut effs, &StaticTimeline::new());
        try_refresh_dirty_stat_attr(&mut attr, &mut effs);

        let mut effs = UpsertContainer::<StatAttrEff<String, TickTimer>>::default();
        clean_expired_element(&mut effs, ());
        try_refresh_dirty_stat_attr(&mut attr, &mut effs);
    }

    /// 一个面向对象的写法样例
    #[test]
    fn example_process_tick() {
        type S = String;
        type AttrEffs = UpsertContainer<StatAttrEff<S, StaticTimer>>;
        let delta: time_type::T = time_type::ZERO;
        let timeline: &mut StaticTimeline = &mut StaticTimeline::default();
        let cleaner: &mut UpsertContainerCleaner = &mut UpsertContainerCleaner::default();
        let attr_effs: &mut [(&mut StatAttr, &mut AttrEffs)] = &mut [];

        // do process_tick

        timeline.0.tick(delta);

        for (attr, effs) in &mut *attr_effs {
            clean_expired_element(effs, timeline);
            try_refresh_dirty_stat_attr(attr, effs);
        }

        // 【规整处理，业务无关】

        // 惰性迭代器
        let ll = attr_effs.iter_mut().map(|(_, effs)| &mut **effs);
        try_clean_hole(delta, ll, cleaner);

        // 基本不会需要重置时间线，实际不用写
        let timers_iter = attr_effs
            .iter_mut()
            .map(|(_, effs)| &mut **effs)
            .flat_map(|effs| effs.iter_mut())
            .map(|eff| eff.get_timer_mut());
        try_reset_timeline(timeline, timers_iter);
    }
}

use crate::base_lib::{
    cores::{
        timers::{
            static_timer::{StaticTimeline, StaticTimer},
            tiny_timer::{HasTimer, TimerView},
        },
        unify_types::{FixedName, time_type},
    },
    eff_attr_prop::{
        attr_eff::AttrEffect,
        attrs::Attr,
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
    use crate::base_lib::{
        cores::timers::{
            tick_timer::TickTimer,
            tiny_timer::{Tickable, TimerProgress},
        },
        eff_attr_prop::{attr_eff::AttrEffectType, effects::Effect},
    };

    use super::*;

    /// 同时支持 [`StaticTimer`] [`TickTimer`]
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

    /// 一个面向对象的写法样例
    #[test]
    fn example_process_tick() {
        type S = String;
        type AttrEffs = UpsertContainer<AttrEffect<S, StaticTimer>>;
        let delta: time_type::T = time_type::ZERO;
        let timeline: &mut StaticTimeline = &mut StaticTimeline::default();
        let cleaner: &mut UpsertContainerCleaner = &mut UpsertContainerCleaner::default();
        let attr_effs: &mut [(&mut Attr, &mut AttrEffs)] = &mut [];

        // do process_tick

        timeline.0.tick(delta);

        for (attr, effs) in &mut *attr_effs {
            clean_expired_element(effs, timeline);
            try_refresh_dirty_attr(attr, effs);
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

    /// 正常帧组合(真实数据):时间线推进 → 过期效果被清理 → 脏属性刷新;规整空洞
    #[test]
    fn real_data_per_entity_tick() {
        type S = String;
        type Effs = UpsertContainer<AttrEffect<S, StaticTimer>>;

        let delta: time_type::T = time_type::unit::<1>();
        let mut timeline = StaticTimeline::new();
        let mut attr = Attr::new(100.0);
        let mut effs = Effs::default();

        // 一个 3s 后过期的效果(基础加法 +20)和一个永不过期的效果(基础加法 +5)
        effs.upsert_ele(
            AttrEffect::new(
                AttrEffectType::BasicAdd,
                Effect::new_form("buff", "short", 20.0),
                StaticTimer::new(&timeline, time_type::unit::<3>()),
            ),
            |_, _| {},
        );
        effs.upsert_ele(
            AttrEffect::new(
                AttrEffectType::BasicAdd,
                Effect::new_form("buff", "inf", 5.0),
                StaticTimer::inf(),
            ),
            |_, _| {},
        );

        // 帧 1:推进时间线 → 清理过期 → 刷新脏属性
        timeline.0.tick(delta);
        clean_expired_element(&mut effs, &timeline);
        try_refresh_dirty_attr(&mut attr, &mut effs);
        assert_eq!(attr.get_current(), 125.0); // 100 + 20 + 5

        // 帧 2/3:推进到 3s,short 过期被清理,inf 保留
        timeline.0.tick(delta);
        timeline.0.tick(delta);
        clean_expired_element(&mut effs, &timeline);
        try_refresh_dirty_attr(&mut attr, &mut effs);
        assert_eq!(attr.get_current(), 105.0); // 100 + 5

        // 规整空洞:周期性清理(累计 delta 超过 5s 周期触发,业务无关)
        let cleaner = &mut UpsertContainerCleaner::default();
        let ll = std::iter::once(&mut effs);
        try_clean_hole(time_type::unit::<6>(), ll, cleaner);
        assert_eq!(effs.ele_len(), 1);
    }

    /// 时间线重置(真实数据):越过一年门槛 → try_reset_timeline → 时间线归零、相对读数不变
    #[test]
    fn real_data_timeline_reset() {
        type S = String;
        type Effs = UpsertContainer<AttrEffect<S, StaticTimer>>;

        let mut timeline = StaticTimeline::new();
        let mut effs = Effs::default();

        // 时间线推进到接近一年(用大 tick 模拟长期运行的漂移累积)
        timeline
            .0
            .tick(time_type::RESET_TIMELINE_PERIOD - time_type::unit::<10>());
        // 一个 30s 时长的效果:重置时仍存活
        effs.upsert_ele(
            AttrEffect::new(
                AttrEffectType::BasicAdd,
                Effect::new_form("buff", "long", 10.0),
                StaticTimer::new(&timeline, time_type::unit::<30>()),
            ),
            |_, _| {},
        );

        // 再推进 20s 越过一年门槛
        timeline.0.tick(time_type::unit::<20>());
        assert!(timeline.current_time() >= time_type::RESET_TIMELINE_PERIOD);

        // 重置前的中飞行计时器相对读数
        let timer = effs.iter_ele().next().unwrap().get_timer();
        let elapsed_before = timer.elapsed(&timeline);
        let remaining_before = timer.remaining(&timeline);
        assert_eq!(remaining_before, time_type::unit::<10>());

        // 重置时间线并修正所有依赖计时器
        let timers_iter = effs.iter_mut().map(|eff| eff.get_timer_mut());
        try_reset_timeline(&mut timeline, timers_iter);

        // 时间线归零,依赖计时器相对读数保持不变
        assert_eq!(timeline.current_time(), time_type::ZERO);
        let timer = effs.iter_ele().next().unwrap().get_timer();
        assert_eq!(timer.elapsed(&timeline), elapsed_before);
        assert_eq!(timer.remaining(&timeline), remaining_before);
    }
}

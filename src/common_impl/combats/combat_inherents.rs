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

/// 内禀属性
///
/// 字段设为 pub 支持拆分平铺到实体中去
pub struct CombatInherentAttr<S: FixedName> {
    /// 气力
    pub strength: Attr,
    /// 信念
    pub belief: Attr,

    /// 气力效果集
    pub strength_effs: UpsertContainer<AttrEffect<S, StaticTimer>>,
    /// 信念效果集
    pub belief_effs: UpsertContainer<AttrEffect<S, StaticTimer>>,

    /// 时间线
    pub timeline: StaticTimeline,

    /// 定时清理
    pub cleaner: UpsertContainerCleaner,
}

impl<S: FixedName> CombatInherentAttr<S> {
    pub fn new(strength: f64, belief: f64) -> Self {
        Self {
            strength: Attr::new(strength),
            belief: Attr::new(belief),
            strength_effs: UpsertContainer::default(),
            belief_effs: UpsertContainer::default(),
            timeline: StaticTimeline::new(),
            cleaner: UpsertContainerCleaner::default(),
        }
    }

    pub fn process_time(&mut self, delta: f64) {
        self.timeline.0.tick(delta);

        Self::try_update_attr(&mut self.strength, &mut self.strength_effs, &self.timeline);
        Self::try_update_attr(&mut self.belief, &mut self.belief_effs, &self.timeline);

        // 规整处理，业务无关

        let should_clean_period = UpsertContainerCleaner::get_default_period();
        let should_clean_hole = self.cleaner.should_clean_hole(delta, should_clean_period);
        if should_clean_hole {
            self.cleaner.do_clean_hole(&mut self.strength_effs);
            self.cleaner.do_clean_hole(&mut self.belief_effs);
        }

        let mut should_restart_timeline = true;
        should_restart_timeline &= self.strength_effs.ele_empty();
        should_restart_timeline &= self.belief_effs.ele_empty();
        if should_restart_timeline {
            self.timeline.restart_timeline();
        }
    }

    fn try_update_attr(
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
}

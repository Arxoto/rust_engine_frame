use crate::base_lib::{
    cores::{
        static_timer::{StaticTimeline, StaticTimer},
        unify_types::FixedName,
    },
    eff_attr_prop::{
        attr_eff::AttrEffect,
        attr_system,
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
        attr_system::process_tick(
            delta,
            &mut self.timeline,
            &mut self.cleaner,
            &mut [
                (&mut self.strength, &mut self.strength_effs),
                (&mut self.belief, &mut self.belief_effs),
            ],
        );
    }
}

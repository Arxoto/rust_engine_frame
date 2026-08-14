use crate::base_lib::{
    cores::{
        timers::static_timer::{StaticTimeline, StaticTimer},
        unify_types::{FixedName, time_type},
    },
    eff_attr_prop::{
        attr_eff::AttrEffect,
        attr_systems,
        attrs::Attr,
        upsert_container::{UpsertContainer, UpsertContainerCleaner},
    },
};

/// 外赋属性
///
/// 字段设为 pub 支持拆分平铺到实体中去
pub struct CombatAdditionAttr<S: FixedName> {
    /// 武器锋利度
    pub weapon_sharp: Attr,
    /// 武器质量
    pub weapon_mass: Attr,
    /// 盔甲坚韧
    pub armor_hard: Attr,
    /// 盔甲柔韧
    pub armor_soft: Attr,
    /// 盔甲质量
    pub armor_mass: Attr,

    pub weapon_sharp_effs: UpsertContainer<AttrEffect<S, StaticTimer>>,
    pub weapon_mass_effs: UpsertContainer<AttrEffect<S, StaticTimer>>,
    pub armor_hard_effs: UpsertContainer<AttrEffect<S, StaticTimer>>,
    pub armor_soft_effs: UpsertContainer<AttrEffect<S, StaticTimer>>,
    pub armor_mass_effs: UpsertContainer<AttrEffect<S, StaticTimer>>,

    /// 时间线
    pub timeline: StaticTimeline,

    /// 定时清理
    pub cleaner: UpsertContainerCleaner,
}

impl<S: FixedName> Default for CombatAdditionAttr<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: FixedName> CombatAdditionAttr<S> {
    pub fn new() -> Self {
        Self {
            weapon_sharp: Attr::new(0.0),
            weapon_mass: Attr::new(0.0),
            armor_hard: Attr::new(0.0),
            armor_soft: Attr::new(0.0),
            armor_mass: Attr::new(0.0),
            weapon_sharp_effs: UpsertContainer::default(),
            weapon_mass_effs: UpsertContainer::default(),
            armor_hard_effs: UpsertContainer::default(),
            armor_soft_effs: UpsertContainer::default(),
            armor_mass_effs: UpsertContainer::default(),
            timeline: StaticTimeline::new(),
            cleaner: UpsertContainerCleaner::default(),
        }
    }

    pub fn process_tick(&mut self, delta: time_type::T) {
        attr_systems::process_tick(
            delta,
            &mut self.timeline,
            &mut self.cleaner,
            &mut [
                (&mut self.weapon_sharp, &mut self.weapon_sharp_effs),
                (&mut self.weapon_mass, &mut self.weapon_mass_effs),
                (&mut self.armor_hard, &mut self.armor_hard_effs),
                (&mut self.armor_soft, &mut self.armor_soft_effs),
                (&mut self.armor_mass, &mut self.armor_mass_effs),
            ],
        );
    }
}

// todo for combat_additions

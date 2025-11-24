use crate::{
    attrs::{dyn_attr::DynAttr, dyn_attr_effect::DynAttrEffect},
    cores::unify_type::FixedName,
    effects::duration_effect::EffectBuilder,
};

/// 外赋属性
pub struct EquipUnit<S: FixedName = String> {
    /// 武器锋利度
    pub(crate) weapon_sharp: DynAttr<S>,
    /// 武器质量 外赋属性（装备加成）
    ///
    /// - 直接影响：连击奖励和冲击伤害
    pub(crate) weapon_mass: DynAttr<S>,
    /// 盔甲坚韧 外赋属性（装备加成）
    ///
    /// - 直接影响：切割伤害
    pub(crate) armor_hard: DynAttr<S>,
    /// 盔甲柔韧 外赋属性（装备加成）
    ///
    /// - 直接影响：冲击伤害
    pub(crate) armor_soft: DynAttr<S>,
}

impl<S: FixedName> EquipUnit<S> {
    pub fn new() -> EquipUnit<S> {
        EquipUnit {
            weapon_sharp: DynAttr::new(0.0),
            weapon_mass: DynAttr::new(0.0),
            armor_hard: DynAttr::new(0.0),
            armor_soft: DynAttr::new(0.0),
        }
    }

    /// 注意函数内创建的效果默认是不允许堆叠的，因此想要实现双持武器时，需要给予不同的效果名称以防止覆盖
    pub fn apply_weapon(&mut self, effect_name: S, weapon_name: S, sharp: f64, mass: f64) {
        self.weapon_sharp.put_or_stack_effect(DynAttrEffect::new(
            crate::attrs::dyn_attr_effect::DynAttrEffectType::BasicAdd,
            EffectBuilder::new_infinite(weapon_name.clone(), effect_name.clone(), sharp),
        ));
        self.weapon_mass.put_or_stack_effect(DynAttrEffect::new(
            crate::attrs::dyn_attr_effect::DynAttrEffectType::BasicAdd,
            EffectBuilder::new_infinite(weapon_name, effect_name, mass),
        ));
    }

    /// 注意函数内创建的效果默认是不允许堆叠的，因此想要实现部位装备时，需要给予不同的效果名称以防止覆盖
    pub fn apply_armor(&mut self, effect_name: S, weapon_name: S, hard: f64, soft: f64) {
        self.armor_hard.put_or_stack_effect(DynAttrEffect::new(
            crate::attrs::dyn_attr_effect::DynAttrEffectType::BasicAdd,
            EffectBuilder::new_infinite(weapon_name.clone(), effect_name.clone(), hard),
        ));
        self.armor_soft.put_or_stack_effect(DynAttrEffect::new(
            crate::attrs::dyn_attr_effect::DynAttrEffectType::BasicAdd,
            EffectBuilder::new_infinite(weapon_name, effect_name, soft),
        ));
    }
}

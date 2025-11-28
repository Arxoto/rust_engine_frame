use crate::{
    attrs::{
        dyn_attr::DynAttr,
        dyn_attr_effect::{DynAttrEffect, DynAttrEffectType},
    },
    cores::unify_type::FixedName,
    effects::duration_effect::EffectBuilder,
};

/// 外赋属性
pub struct CombatAdditionAttr<S: FixedName> {
    /// 武器锋利度
    pub(crate) weapon_sharp: DynAttr<S>,
    /// 武器质量
    pub(crate) weapon_mass: DynAttr<S>,
    /// 盔甲坚韧
    pub(crate) armor_hard: DynAttr<S>,
    /// 盔甲柔韧
    pub(crate) armor_soft: DynAttr<S>,
    /// 盔甲质量
    pub(crate) armor_mass: DynAttr<S>,
}

impl<S: FixedName> CombatAdditionAttr<S> {
    pub fn new() -> CombatAdditionAttr<S> {
        CombatAdditionAttr {
            weapon_sharp: DynAttr::new(0.0),
            weapon_mass: DynAttr::new(0.0),
            armor_hard: DynAttr::new(0.0),
            armor_soft: DynAttr::new(0.0),
            armor_mass: DynAttr::new(0.0),
        }
    }

    /// 注意函数内创建的效果默认是不允许堆叠的，因此想要实现双持武器时，需要给予不同的效果名称以防止覆盖
    pub fn apply_weapon(&mut self, effect_name: &S, weapon_name: &S, sharp: f64, mass: f64) {
        self.weapon_sharp.put_or_stack_effect(DynAttrEffect::new(
            DynAttrEffectType::BasicAdd,
            EffectBuilder::new_infinite(weapon_name.clone(), effect_name.clone(), sharp),
        ));
        self.weapon_mass.put_or_stack_effect(DynAttrEffect::new(
            DynAttrEffectType::BasicAdd,
            EffectBuilder::new_infinite(weapon_name.clone(), effect_name.clone(), mass),
        ));
    }

    /// 注意函数内创建的效果默认是不允许堆叠的，因此想要实现部位装备时，需要给予不同的效果名称以防止覆盖
    pub fn apply_armor(
        &mut self,
        effect_name: &S,
        weapon_name: &S,
        hard: f64,
        soft: f64,
        mass: f64,
    ) {
        self.armor_hard.put_or_stack_effect(DynAttrEffect::new(
            DynAttrEffectType::BasicAdd,
            EffectBuilder::new_infinite(weapon_name.clone(), effect_name.clone(), hard),
        ));
        self.armor_soft.put_or_stack_effect(DynAttrEffect::new(
            DynAttrEffectType::BasicAdd,
            EffectBuilder::new_infinite(weapon_name.clone(), effect_name.clone(), soft),
        ));
        self.armor_mass.put_or_stack_effect(DynAttrEffect::new(
            DynAttrEffectType::BasicAdd,
            EffectBuilder::new_infinite(weapon_name.clone(), effect_name.clone(), mass),
        ));
    }
}

impl<S: FixedName> Default for CombatAdditionAttr<S> {
    fn default() -> Self {
        Self::new()
    }
}

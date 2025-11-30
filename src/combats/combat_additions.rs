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

impl<S: FixedName> Default for CombatAdditionAttr<S> {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn process_time(&mut self, delta: f64) {
        self.weapon_sharp.process_time(delta);
        self.weapon_mass.process_time(delta);
        self.armor_hard.process_time(delta);
        self.armor_soft.process_time(delta);
        self.armor_mass.process_time(delta);
    }

    /// 注意函数内创建的效果默认是不允许堆叠的，因此想要实现双持武器时，需要给予不同的效果名称以防止覆盖
    pub(crate) fn apply_weapon(&mut self, effect_name: &S, weapon_name: &S, sharp: f64, mass: f64) {
        self.weapon_sharp.put_or_stack_effect(DynAttrEffect::new(
            DynAttrEffectType::BasicAdd,
            EffectBuilder::new_infinite(weapon_name.clone(), effect_name.clone(), sharp),
        ));
        self.weapon_sharp.refresh_value();

        self.weapon_mass.put_or_stack_effect(DynAttrEffect::new(
            DynAttrEffectType::BasicAdd,
            EffectBuilder::new_infinite(weapon_name.clone(), effect_name.clone(), mass),
        ));
        self.weapon_mass.refresh_value();
    }

    /// 注意函数内创建的效果默认是不允许堆叠的，因此想要实现部位装备时，需要给予不同的效果名称以防止覆盖
    pub(crate) fn apply_armor(
        &mut self,
        effect_name: &S,
        armor_name: &S,
        hard: f64,
        soft: f64,
        mass: f64,
    ) {
        self.armor_hard.put_or_stack_effect(DynAttrEffect::new(
            DynAttrEffectType::BasicAdd,
            EffectBuilder::new_infinite(armor_name.clone(), effect_name.clone(), hard),
        ));
        self.armor_hard.refresh_value();

        self.armor_soft.put_or_stack_effect(DynAttrEffect::new(
            DynAttrEffectType::BasicAdd,
            EffectBuilder::new_infinite(armor_name.clone(), effect_name.clone(), soft),
        ));
        self.armor_soft.refresh_value();

        self.armor_mass.put_or_stack_effect(DynAttrEffect::new(
            DynAttrEffectType::BasicAdd,
            EffectBuilder::new_infinite(armor_name.clone(), effect_name.clone(), mass),
        ));
        self.armor_mass.refresh_value();
    }

    /// 注意函数内创建的效果默认是不允许堆叠的，因此想要实现双持武器时，需要给予不同的效果名称以防止覆盖
    pub fn apply_equip_weapon<T: Into<S>>(
        &mut self,
        effect_name: T,
        equip_weapon: &CombatEquipWeapon<S>,
    ) {
        self.apply_weapon(
            &effect_name.into(),
            &equip_weapon.weapon_name,
            equip_weapon.weapon_sharp,
            equip_weapon.weapon_mass,
        );
    }

    /// 注意函数内创建的效果默认是不允许堆叠的，因此想要实现部位装备时，需要给予不同的效果名称以防止覆盖
    pub fn apply_equip_armor<T: Into<S>>(
        &mut self,
        effect_name: T,
        equip_armor: &CombatEquipArmor<S>,
    ) {
        self.apply_armor(
            &effect_name.into(),
            &equip_armor.armor_name,
            equip_armor.armor_hard,
            equip_armor.armor_soft,
            equip_armor.armor_mass,
        );
    }
}

pub struct CombatEquipWeapon<S: FixedName> {
    pub(crate) weapon_name: S,
    /// 武器锋利度
    pub(crate) weapon_sharp: f64,
    /// 武器质量
    pub(crate) weapon_mass: f64,
}

impl<S: FixedName> CombatEquipWeapon<S> {
    pub fn new<T: Into<S>>(
        weapon_name: T,
        weapon_sharp: f64,
        weapon_mass: f64,
    ) -> CombatEquipWeapon<S> {
        let weapon_name: S = weapon_name.into();
        CombatEquipWeapon {
            weapon_name,
            weapon_sharp,
            weapon_mass,
        }
    }
}

pub struct CombatEquipArmor<S: FixedName> {
    pub(crate) armor_name: S,
    /// 盔甲坚韧
    pub(crate) armor_hard: f64,
    /// 盔甲柔韧
    pub(crate) armor_soft: f64,
    /// 盔甲质量
    pub(crate) armor_mass: f64,
}

impl<S: FixedName> CombatEquipArmor<S> {
    pub fn new<T: Into<S>>(
        armor_name: T,
        armor_hard: f64,
        armor_soft: f64,
        armor_mass: f64,
    ) -> CombatEquipArmor<S> {
        let armor_name: S = armor_name.into();
        CombatEquipArmor {
            armor_name,
            armor_hard,
            armor_soft,
            armor_mass,
        }
    }
}

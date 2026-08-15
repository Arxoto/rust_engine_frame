//! 战斗相关的装备（武器、盔甲），直接影响【外赋属性】

use crate::{
    base_lib::{cores::unify_types::FixedName, eff_attr_prop::upsert_container::Upsert},
    common_impl::combats::combat_additions::{
        ArmorHardEffs, ArmorMassEffs, ArmorSoftEffs, WeaponMassEffs, WeaponSharpEffs,
    },
};

pub struct EquipWeapon<S: FixedName> {
    name: S,
    /// 武器锋利度
    sharp: f64,
    /// 武器质量
    mass: f64,
}

pub struct EquipArmor<S: FixedName> {
    name: S,
    /// 盔甲坚韧
    hard: f64,
    /// 盔甲柔韧
    soft: f64,
    /// 盔甲质量
    mass: f64,
}

impl<S: FixedName> EquipWeapon<S> {
    pub fn new(name: S, sharp: f64, mass: f64) -> Self {
        Self { name, sharp, mass }
    }

    pub fn new_from<T: Into<S>>(name: T, sharp: f64, mass: f64) -> Self {
        let name: S = name.into();
        Self { name, sharp, mass }
    }

    pub fn equip(
        &self,
        char_name: &S,
        weapon_sharp_effs: &mut WeaponSharpEffs<S>,
        weapon_mass_effs: &mut WeaponMassEffs<S>,
    ) {
        equip_system::add_attr_eff(char_name, &self.name, self.sharp, &mut weapon_sharp_effs.0);
        equip_system::add_attr_eff(char_name, &self.name, self.mass, &mut weapon_mass_effs.0);
    }

    pub fn take_off(
        &self,
        char_name: &S,
        weapon_sharp_effs: &mut WeaponSharpEffs<S>,
        weapon_mass_effs: &mut WeaponMassEffs<S>,
    ) {
        let eff_id = equip_system::gen_attr_eff_id(char_name, &self.name);
        weapon_sharp_effs.0.delete_ele(|e| e.matched_id(&eff_id));
        weapon_mass_effs.0.delete_ele(|e| e.matched_id(&eff_id));
    }
}

impl<S: FixedName> EquipArmor<S> {
    pub fn new(name: S, hard: f64, soft: f64, mass: f64) -> Self {
        Self {
            name,
            hard,
            soft,
            mass,
        }
    }

    pub fn new_from<T: Into<S>>(name: T, hard: f64, soft: f64, mass: f64) -> Self {
        let name: S = name.into();
        Self {
            name,
            hard,
            soft,
            mass,
        }
    }

    pub fn equip(
        &self,
        char_name: &S,
        armor_hard_effs: &mut ArmorHardEffs<S>,
        armor_soft_effs: &mut ArmorSoftEffs<S>,
        armor_mass_effs: &mut ArmorMassEffs<S>,
    ) {
        equip_system::add_attr_eff(char_name, &self.name, self.hard, &mut armor_hard_effs.0);
        equip_system::add_attr_eff(char_name, &self.name, self.soft, &mut armor_soft_effs.0);
        equip_system::add_attr_eff(char_name, &self.name, self.mass, &mut armor_mass_effs.0);
    }

    pub fn take_off(
        &self,
        char_name: &S,
        armor_hard_effs: &mut ArmorHardEffs<S>,
        armor_soft_effs: &mut ArmorSoftEffs<S>,
        armor_mass_effs: &mut ArmorMassEffs<S>,
    ) {
        let attr_eff_id = equip_system::gen_attr_eff_id(char_name, &self.name);
        armor_hard_effs.0.delete_ele(|e| e.matched_id(&attr_eff_id));
        armor_soft_effs.0.delete_ele(|e| e.matched_id(&attr_eff_id));
        armor_mass_effs.0.delete_ele(|e| e.matched_id(&attr_eff_id));
    }
}

mod equip_system {
    use crate::base_lib::{
        cores::{timers::static_timer::StaticTimer, unify_types::FixedName},
        eff_attr_prop::{
            attr_eff::{AttrEffId, AttrEffect, AttrEffectType},
            effects::Effect,
            upsert_container::{Upsert, UpsertContainer},
        },
    };

    /// id 与 [`add_attr_eff`] 中 eff 赋值逻辑保持一致
    pub fn gen_attr_eff_id<S: FixedName>(char_name: &S, equip_name: &S) -> AttrEffId<S> {
        AttrEffId {
            eff: equip_name.clone(),
            from: char_name.clone(),
        }
    }

    /// 触发装备的属性效果
    ///
    /// 由于 [`AttrEffect`] 是不允许堆叠的，因此若要实现双持武器，设置不同的装备名称
    pub fn add_attr_eff<S: FixedName>(
        char_name: &S,
        equip_name: &S,
        equip_value: f64,
        effs: &mut UpsertContainer<AttrEffect<S, StaticTimer>>,
    ) {
        let attr_eff = AttrEffect::new(
            AttrEffectType::BasicAdd,
            Effect::new(char_name.clone(), equip_name.clone(), equip_value),
            StaticTimer::inf(),
        );
        effs.upsert_ele(attr_eff, |old, new| {
            Upsert::replace(old, new);
        });
    }
}

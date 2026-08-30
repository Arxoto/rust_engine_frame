//! 战斗相关的装备（武器、盔甲），直接影响【外赋属性】

use slotmap::DefaultKey;

use crate::{
    base_lib::{
        cores::unify_types::FixedName,
        eff_attr::{
            modifier_collections::ModifierCollection,
            stat_attr_modifiers::{StatAttrModifier, StatAttrModifyDimension},
        },
    },
    common_impl::combats::combat_additions::{
        ArmorHardEffs, ArmorMassEffs, ArmorSoftEffs, WeaponMassEffs, WeaponSharpEffs,
    },
};

/// 根据武器生成的数据集
pub struct EquipWeapon<S: FixedName> {
    pub name: S,
    /// 武器锋利度
    pub sharp: f64,
    /// 武器质量
    pub mass: f64,
}

/// 根据盔甲生成的数据集
pub struct EquipArmor<S: FixedName> {
    pub name: S,
    /// 盔甲坚韧
    pub hard: f64,
    /// 盔甲柔韧
    pub soft: f64,
    /// 盔甲质量
    pub mass: f64,
}

/// 角色装备的武器槽
///
/// 若要实现双持武器，角色实体需要持有多个 [`EquipWeaponSlot`]
#[derive(Debug, Default)]
pub struct EquipWeaponSlot {
    sharp: Option<DefaultKey>,
    mass: Option<DefaultKey>,
}

/// 角色装备的盔甲槽
///
/// 弱要实现部位装备，角色实体需要持有多个 [`EquipArmorSlot`]
#[derive(Debug, Default)]
pub struct EquipArmorSlot {
    hard: Option<DefaultKey>,
    soft: Option<DefaultKey>,
    mass: Option<DefaultKey>,
}

impl EquipWeaponSlot {
    pub fn equip<S: FixedName>(
        &mut self,
        weapon: EquipWeapon<S>,
        weapon_sharp_effs: &mut WeaponSharpEffs,
        weapon_mass_effs: &mut WeaponMassEffs,
    ) {
        add_attr_eff(weapon.sharp, &mut self.sharp, &mut weapon_sharp_effs.0);
        add_attr_eff(weapon.mass, &mut self.mass, &mut weapon_mass_effs.0);
    }

    pub fn take_off(
        &mut self,
        weapon_sharp_effs: &mut WeaponSharpEffs,
        weapon_mass_effs: &mut WeaponMassEffs,
    ) {
        rm_attr_eff(&mut self.sharp, &mut weapon_sharp_effs.0);
        rm_attr_eff(&mut self.mass, &mut weapon_mass_effs.0);
    }
}

impl EquipArmorSlot {
    pub fn equip<S: FixedName>(
        &mut self,
        armor: &EquipArmor<S>,
        armor_hard_effs: &mut ArmorHardEffs,
        armor_soft_effs: &mut ArmorSoftEffs,
        armor_mass_effs: &mut ArmorMassEffs,
    ) {
        add_attr_eff(armor.hard, &mut self.hard, &mut armor_hard_effs.0);
        add_attr_eff(armor.soft, &mut self.soft, &mut armor_soft_effs.0);
        add_attr_eff(armor.mass, &mut self.mass, &mut armor_mass_effs.0);
    }

    pub fn take_off(
        &mut self,
        armor_hard_effs: &mut ArmorHardEffs,
        armor_soft_effs: &mut ArmorSoftEffs,
        armor_mass_effs: &mut ArmorMassEffs,
    ) {
        rm_attr_eff(&mut self.hard, &mut armor_hard_effs.0);
        rm_attr_eff(&mut self.soft, &mut armor_soft_effs.0);
        rm_attr_eff(&mut self.mass, &mut armor_mass_effs.0);
    }
}

/// 触发装备的属性效果
#[inline]
fn add_attr_eff(
    equip_value: f64,
    key: &mut Option<DefaultKey>,
    effs: &mut ModifierCollection<StatAttrModifier>,
) {
    if let Some(key) = key {
        effs.remove(*key);
    }
    let attr_eff = StatAttrModifier::new(StatAttrModifyDimension::BasicAdd, equip_value);
    *key = Some(effs.insert(attr_eff));
}

#[inline]
fn rm_attr_eff(key: &mut Option<DefaultKey>, effs: &mut ModifierCollection<StatAttrModifier>) {
    if let Some(key) = key {
        effs.remove(*key);
    }
    *key = None;
}

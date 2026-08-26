//! 战斗相关的装备（武器、盔甲），直接影响【外赋属性】

use crate::{
    base_lib::{cores::unify_types::FixedName, eff_attr::upsert_container::Upsert},
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
        let eff_id_ref = equip_system::gen_attr_eff_id(char_name, &self.name);
        weapon_sharp_effs.0.delete_ele(|e| e.id_ref() == eff_id_ref);
        weapon_mass_effs.0.delete_ele(|e| e.id_ref() == eff_id_ref);
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
        let eff_id_ref = equip_system::gen_attr_eff_id(char_name, &self.name);
        armor_hard_effs.0.delete_ele(|e| e.id_ref() == eff_id_ref);
        armor_soft_effs.0.delete_ele(|e| e.id_ref() == eff_id_ref);
        armor_mass_effs.0.delete_ele(|e| e.id_ref() == eff_id_ref);
    }
}

mod equip_system {
    use crate::base_lib::{
        cores::{timers::static_timer::StaticTimer, unify_types::FixedName},
        eff_attr::{
            effects::{EffIdRef, Effect},
            stat_attr_effs::{StatAttrEff, StatAttrEffType},
            upsert_container::UpsertContainer,
        },
    };

    /// id 与 [`add_attr_eff`] 中 eff 赋值逻辑保持一致
    pub fn gen_attr_eff_id<'a, S: FixedName>(
        char_name: &'a S,
        equip_name: &'a S,
    ) -> EffIdRef<'a, S> {
        EffIdRef {
            from_name: char_name,
            effect_name: equip_name,
        }
    }

    /// 触发装备的属性效果
    ///
    /// 由于 [`AttrEffect`] 是不允许堆叠的，因此若要实现双持武器，设置不同的装备名称
    pub fn add_attr_eff<S: FixedName>(
        char_name: &S,
        equip_name: &S,
        equip_value: f64,
        effs: &mut UpsertContainer<StatAttrEff<S, StaticTimer>>,
    ) {
        let attr_eff = StatAttrEff::new(
            StatAttrEffType::BasicAdd,
            Effect::new(char_name.clone(), equip_name.clone(), equip_value),
            StaticTimer::inf(),
        );
        effs.upsert_replace(attr_eff);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base_lib::eff_attr::{
            attr_systems::try_refresh_dirty_stat_attr, stat_attrs::StatAttr,
            upsert_container::UpsertContainer,
        },
        common_impl::combats::combat_additions::{
            ArmorHard, ArmorMass, ArmorSoft, WeaponMass, WeaponSharp,
        },
    };

    /// equip → 刷新 → 外赋属性生效;take_off → 刷新 → 回落
    ///
    /// 说明:本测试仅含装备的 inf 效果(永不结束),故无需 StaticTimeline 驱动;
    /// 业务上其他有限时长效果仍需时间线 + clean_expired_element,与本测试无关。
    #[test]
    fn equip_weapon_chain_reflects_to_attr() {
        let mut sharp_attr = WeaponSharp(StatAttr::new(10.0));
        let mut mass_attr = WeaponMass(StatAttr::new(5.0));
        let mut sharp_effs: WeaponSharpEffs<String> = WeaponSharpEffs(UpsertContainer::default());
        let mut mass_effs: WeaponMassEffs<String> = WeaponMassEffs(UpsertContainer::default());

        let weapon = EquipWeapon::new("iron_sword".to_string(), 8.0, 3.0);
        let char_name = "player".to_string();

        // 穿上:写入外赋属性效果 → 刷新
        weapon.equip(&char_name, &mut sharp_effs, &mut mass_effs);
        try_refresh_dirty_stat_attr(&mut sharp_attr.0, &mut sharp_effs.0);
        try_refresh_dirty_stat_attr(&mut mass_attr.0, &mut mass_effs.0);

        assert_eq!(sharp_attr.0.get_current(), 18.0); // 10 + 锋利 8
        assert_eq!(mass_attr.0.get_current(), 8.0); // 5 + 质量 3

        // 脱掉:删除效果 → 刷新 → 回落
        weapon.take_off(&char_name, &mut sharp_effs, &mut mass_effs);
        try_refresh_dirty_stat_attr(&mut sharp_attr.0, &mut sharp_effs.0);
        try_refresh_dirty_stat_attr(&mut mass_attr.0, &mut mass_effs.0);

        assert_eq!(sharp_attr.0.get_current(), 10.0);
        assert_eq!(mass_attr.0.get_current(), 5.0);
    }

    /// EquipArmor 链:equip → 刷新 → 坚韧/柔韧/质量生效;take_off → 回落
    #[test]
    fn equip_armor_chain_reflects_to_attr() {
        let mut hard_attr = ArmorHard(StatAttr::new(20.0));
        let mut soft_attr = ArmorSoft(StatAttr::new(10.0));
        let mut mass_attr = ArmorMass(StatAttr::new(5.0));
        let mut hard_effs: ArmorHardEffs<String> = ArmorHardEffs(UpsertContainer::default());
        let mut soft_effs: ArmorSoftEffs<String> = ArmorSoftEffs(UpsertContainer::default());
        let mut mass_effs: ArmorMassEffs<String> = ArmorMassEffs(UpsertContainer::default());

        let armor = EquipArmor::new("plate".to_string(), 30.0, 6.0, 2.0);
        let char_name = "player".to_string();

        armor.equip(&char_name, &mut hard_effs, &mut soft_effs, &mut mass_effs);
        try_refresh_dirty_stat_attr(&mut hard_attr.0, &mut hard_effs.0);
        try_refresh_dirty_stat_attr(&mut soft_attr.0, &mut soft_effs.0);
        try_refresh_dirty_stat_attr(&mut mass_attr.0, &mut mass_effs.0);

        assert_eq!(hard_attr.0.get_current(), 50.0); // 20 + 坚韧 30
        assert_eq!(soft_attr.0.get_current(), 16.0); // 10 + 柔韧 6
        assert_eq!(mass_attr.0.get_current(), 7.0); // 5 + 质量 2

        armor.take_off(&char_name, &mut hard_effs, &mut soft_effs, &mut mass_effs);
        try_refresh_dirty_stat_attr(&mut hard_attr.0, &mut hard_effs.0);
        try_refresh_dirty_stat_attr(&mut soft_attr.0, &mut soft_effs.0);
        try_refresh_dirty_stat_attr(&mut mass_attr.0, &mut mass_effs.0);

        assert_eq!(hard_attr.0.get_current(), 20.0);
        assert_eq!(soft_attr.0.get_current(), 10.0);
        assert_eq!(mass_attr.0.get_current(), 5.0);
    }
}

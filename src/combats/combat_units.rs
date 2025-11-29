use std::ops::Not;

use crate::{
    attrs::{
        dyn_prop::DynProp, dyn_prop_dur_effect::DynPropDurEffect,
        dyn_prop_inst_effect::DynPropInstEffect, dyn_prop_period_effect::DynPropPeriodEffect,
        event_prop::DynPropAlterResult,
    },
    combat::{
        combat_additions::CombatAdditionAttr,
        combat_inherents::CombatInherentAttr,
        damages::{DamageInfo, DamageType, MagickaEnergyLevel, NumericalBalancer},
    },
    cores::unify_type::FixedName,
    effects::{
        duration_effect::EffectBuilder,
        native_duration::ProxyDuration,
        native_effect::{Effect, ProxyEffect},
    },
};

/// 战斗属性
///
/// 参考经典三维：
/// - health 血量
/// - stamina 耐力，这里置换为平衡
/// - magicka/mana 法力，这里置换为能量（气势）
pub struct CombatUnit<S: FixedName> {
    /// 血量和护盾
    pub(crate) health_shields: CombatHealthShield<S>,

    /// 能量（气势）
    pub(crate) magicka: DynProp<S>,

    /// 平衡 战时动态值 变化由冲击韧性系统控制
    ///
    /// 基础值和最大值固定 清空时触发倒地
    pub(crate) stamina: DynProp<S>,

    /// 熵（炎热寒冷） 累积进度条 受元素系统控制
    ///
    /// 为统一区分增益效果的，编码时统一规定所有属性条满时正常，归零异常，满时异常的累积进度条仅体现在视觉表达上
    pub(crate) bar_entropy: DynProp<S>,
    /// 电势能 累积进度条 受元素系统控制
    ///
    /// 为统一区分增益效果的，编码时统一规定所有属性条满时正常，归零异常，满时异常的累积进度条仅体现在视觉表达上
    pub(crate) bar_electric: DynProp<S>,

    /// 内禀属性
    pub(crate) inherent_attr: CombatInherentAttr<S>,
    /// 外赋属性
    pub(crate) addition_attr: CombatAdditionAttr<S>,
}

impl<S: FixedName> CombatUnit<S> {
    pub fn new(
        health_base: f64,
        health_scale: f64,
        magicka_base: f64,
        magicka_scale: f64,
        inherent_attr: CombatInherentAttr<S>,
        addition_attr: CombatAdditionAttr<S>,
        magicka_energy_level: &MagickaEnergyLevel,
    ) -> CombatUnit<S> {
        let health_value =
            NumericalBalancer::calc_health_max(health_base, health_scale, &inherent_attr);

        let magicka_max = NumericalBalancer::calc_magicka_max(
            magicka_base,
            magicka_scale,
            &inherent_attr,
            &magicka_energy_level,
        );

        CombatUnit {
            health_shields: CombatHealthShield {
                health: DynProp::new_by_max(health_value),
                shield_substitute: DynProp::new_by_max(0.0),
                shield_defence: DynProp::new_by_max(0.0),
                shield_arcane: DynProp::new_by_max(0.0),
            },

            magicka: DynProp::new(0.0, magicka_max, 0.0),
            stamina: DynProp::new_by_max(NumericalBalancer::get_default_prop_value()),

            bar_entropy: DynProp::new_by_max(NumericalBalancer::get_default_prop_value()),
            bar_electric: DynProp::new_by_max(NumericalBalancer::get_default_prop_value()),

            inherent_attr,
            addition_attr,
        }
    }

    // todo test
    /// 生命值以最大值的百分比进行恢复
    pub fn init_health_eff<T: Into<S>>(
        &mut self,
        from_name: T,
        health_recover_name: T,
        health_recover_ratio: f64,
        health_recover_period: f64,
    ) {
        let e = DynPropPeriodEffect::new_max_per(
            EffectBuilder::new_infinite(from_name, health_recover_name, health_recover_ratio),
            health_recover_period,
        );
        self.health_shields.health.put_period_effect(e);
    }

    /// 平衡以固定值进行恢复 具有延迟时间
    pub fn init_stamina_eff<T: Into<S>>(
        &mut self,
        from_name: T,
        stamina_recover_name: T,
        stamina_recover_value: f64,
        stamina_recover_period: f64,
        stamina_recover_wait: f64,
    ) {
        let mut e = DynPropPeriodEffect::new_val(
            EffectBuilder::new_infinite(from_name, stamina_recover_name, stamina_recover_value),
            stamina_recover_period,
        );
        e.set_wait_time(stamina_recover_wait);
        self.stamina.put_period_effect(e);
    }

    /// 能量以固定值削减 具有延迟时间
    pub fn init_magicka_eff<T: Into<S>>(
        &mut self,
        from_name: T,
        magicka_decline_name: T,
        magicka_decline_value: f64,
        magicka_decline_period: f64,
        magicka_decline_wait: f64,
    ) {
        let mut e = DynPropPeriodEffect::new_val(
            EffectBuilder::new_infinite(from_name, magicka_decline_name, magicka_decline_value),
            magicka_decline_period,
        );
        e.set_wait_time(magicka_decline_wait);
        self.magicka.put_period_effect(e);
    }

    /// 根据外赋属性生成护盾
    pub fn init_addition_eff<T: Into<S>>(&mut self, from_name: T, effect_name: T) {
        let defence_shield_value = NumericalBalancer::calc_defence_shield(&self.addition_attr);
        let eff = DynPropDurEffect::new_max_val(EffectBuilder::new_infinite(
            from_name,
            effect_name,
            defence_shield_value,
        ));
        self.health_shields
            .shield_defence
            .put_and_do_dur_effect(eff);
    }

    /// 造成伤害
    pub fn hurt_health(
        &mut self,
        from_unit: &CombatUnit<S>,
        damage_type: DamageType,
        damage_eff: DynPropInstEffect<S>,
    ) -> DamageInfo {
        let damage_scale = NumericalBalancer::calc_damage_scale(&damage_type, from_unit, self);
        self.health_shields
            .hurt_external(damage_type, damage_eff, damage_scale)
    }

    /// 花费能量
    pub fn cost_magicka(&mut self, eff: DynPropInstEffect<S>) -> DynPropAlterResult {
        self.magicka.use_inst_effect(eff)
    }

    /// 尝试花费能量
    pub fn try_cost_magicka(&mut self, eff: DynPropInstEffect<S>) -> Option<DynPropAlterResult> {
        self.magicka.use_inst_effect_if_enough(eff, 0.0)
    }

    /// 削韧 冲击-平衡
    pub fn cut_stamina(&mut self, eff: DynPropInstEffect<S>) -> DynPropAlterResult {
        self.stamina.use_inst_effect(eff)
    }

    pub fn give_entropy(&mut self, eff: DynPropInstEffect<S>) -> DynPropAlterResult {
        self.bar_entropy.use_inst_effect(eff)
    }

    pub fn give_electric(&mut self, eff: DynPropInstEffect<S>) -> DynPropAlterResult {
        self.bar_electric.use_inst_effect(eff)
    }
}

pub(crate) struct CombatHealthShield<S: FixedName> {
    /// 血量 战时动态值 变化由伤害系统控制
    ///
    /// 基础值受气力的基础值影响
    pub(crate) health: DynProp<S>,

    /// 替身 护盾 受伤害系统控制
    pub(crate) shield_substitute: DynProp<S>,
    /// 防护 护盾 受伤害系统控制
    pub(crate) shield_defence: DynProp<S>,
    /// 奥术 护盾 受伤害系统控制
    pub(crate) shield_arcane: DynProp<S>,
}

impl<S: FixedName> CombatHealthShield<S> {
    fn hurt_internal(mut eff: Effect<S>, props: &mut [&mut DynProp<S>]) -> DamageInfo {
        let mut dead_info = DamageInfo {
            broken: false,
            damage: eff.value,
        };

        for prop in props {
            let alter_result = prop.alter_current_value(&eff);
            let dead = prop.alter_to_min(&alter_result);

            dead_info.broken = dead;
            if dead_info.broken.not() {
                break;
            }
            eff.set_value(eff.get_value() - alter_result.delta);
        }

        dead_info
    }

    // todo test
    pub(crate) fn hurt_external(
        &mut self,
        damage_type: DamageType,
        damage_eff: DynPropInstEffect<S>,
        damage_scale: f64,
    ) -> DamageInfo {
        let base_prop = match damage_type {
            DamageType::KarmaTruth => &self.health,
            DamageType::PhysicsShear => &self.health,
            DamageType::PhysicsImpact => &self.health,
            DamageType::MagickaArcane => &self.health,
            DamageType::BrokeShieldDefence => &self.shield_defence,
            DamageType::BrokeShieldArcane => &self.shield_arcane,
        };
        let mut eff = damage_eff.convert_real_effect(base_prop);
        eff.value *= damage_scale;

        let ll: &mut [&mut DynProp<S>] = match damage_type {
            DamageType::KarmaTruth => &mut [&mut self.health],
            DamageType::PhysicsShear => &mut [
                &mut self.shield_defence,
                &mut self.shield_substitute,
                &mut self.health,
            ],
            DamageType::PhysicsImpact => &mut [&mut self.shield_substitute, &mut self.health],
            DamageType::MagickaArcane => &mut [
                &mut self.shield_arcane,
                &mut self.shield_substitute,
                &mut self.health,
            ],
            DamageType::BrokeShieldDefence => &mut [&mut self.shield_defence],
            DamageType::BrokeShieldArcane => &mut [&mut self.shield_arcane],
        };
        Self::hurt_internal(eff, ll)
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::combat::combat_additions::{CombatEquipArmor, CombatEquipWeapon};

    use super::*;

    #[test]
    fn combat_unit_new_and_init_addition_eff() {
        let strength = 60.0;
        let belief = 80.0;
        let magicka_energy_level = &MagickaEnergyLevel::new(100.0, 200.0, 300.0);

        let health_base = 20.0;
        let health_scale = 1.0;
        let magicka_base = 50.0;
        let magicka_scale = 3.0;

        let armor_hard = 10.0;
        let armor_soft = 10.0;
        let armor_mass = 10.0;

        let weapon_sharp = 1.0;
        let weapon_mass = 5.0;

        let combat_inherent_attr = CombatInherentAttr::new(strength, belief);
        assert_eq!(combat_inherent_attr.strength.get_current(), strength);
        assert_eq!(combat_inherent_attr.belief.get_current(), belief);

        let combat_addition_attr = {
            let mut combat_addition_attr = CombatAdditionAttr::new();
            combat_addition_attr.apply_equip_armor(
                "armor_eff",
                &CombatEquipArmor::new("armor_name", armor_hard, armor_soft, armor_mass),
            );
            combat_addition_attr.apply_equip_weapon(
                "weapon_eff",
                &CombatEquipWeapon::new("weapon_name", weapon_sharp, weapon_mass),
            );
            combat_addition_attr
        };
        assert_eq!(combat_addition_attr.armor_hard.get_current(), armor_hard);
        assert_eq!(combat_addition_attr.armor_soft.get_current(), armor_soft);
        assert_eq!(combat_addition_attr.armor_mass.get_current(), armor_mass);
        assert_eq!(
            combat_addition_attr.weapon_sharp.get_current(),
            weapon_sharp
        );
        assert_eq!(combat_addition_attr.weapon_mass.get_current(), weapon_mass);

        let combat_unit = {
            let mut combat_unit: CombatUnit<&'static str> = CombatUnit::new(
                health_base,
                health_scale,
                magicka_base,
                magicka_scale,
                combat_inherent_attr,
                combat_addition_attr,
                magicka_energy_level,
            );
            combat_unit.init_addition_eff("from_name", "effect_name");
            combat_unit
        };

        assert_eq!(
            combat_unit.health_shields.health.get_current(),
            NumericalBalancer::calc_health_max(
                health_base,
                health_scale,
                &combat_unit.inherent_attr
            )
        );

        assert_eq!(
            combat_unit.health_shields.shield_defence.get_current(),
            NumericalBalancer::calc_defence_shield(&combat_unit.addition_attr)
        );

        assert_eq!(combat_unit.magicka.get_current(), 0.0);
        assert_eq!(
            combat_unit.magicka.get_max(),
            NumericalBalancer::calc_magicka_max(
                magicka_base,
                magicka_scale,
                &combat_unit.inherent_attr,
                magicka_energy_level
            )
        );

        assert_eq!(
            combat_unit.stamina.get_current(),
            NumericalBalancer::get_default_prop_value()
        );
    }
}

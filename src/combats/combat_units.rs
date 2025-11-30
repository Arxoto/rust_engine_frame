use crate::{
    attrs::{
        dyn_prop::DynProp, dyn_prop_dur_effect::DynPropDurEffect,
        dyn_prop_inst_effect::DynPropInstEffect, dyn_prop_period_effect::DynPropPeriodEffect,
        event_prop::DynPropAlterResult,
    },
    combats::{
        combat_additions::CombatAdditionAttr,
        combat_inherents::CombatInherentAttr,
        damages::{DamageInfo, DamageType, MagickaEnergyLevel, NumericalBalancer},
    },
    cores::unify_type::FixedName,
    effects::{
        duration_effect::EffectBuilder, native_duration::ProxyDuration, native_effect::Effect,
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
            magicka_energy_level,
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

    pub fn add_arcane_shield_eff(&mut self, eff: DynPropDurEffect<S>) {
        self.health_shields.shield_arcane.put_and_do_dur_effect(eff);
    }

    pub fn add_substitute_shield_eff(&mut self, eff: DynPropDurEffect<S>) {
        self.health_shields
            .shield_substitute
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

    pub fn process_time(&mut self, delta: f64) {
        self.health_shields.process_time(delta);
        self.magicka.process_time(delta);
        self.stamina.process_time(delta);

        self.bar_entropy.process_time(delta);
        self.bar_electric.process_time(delta);

        self.inherent_attr.process_time(delta);
        self.addition_attr.process_time(delta);
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
    pub fn process_time(&mut self, delta: f64) {
        self.health.process_time(delta);
        self.shield_substitute.process_time(delta);
        self.shield_defence.process_time(delta);
        self.shield_arcane.process_time(delta);
    }

    fn hurt_internal(mut eff: Effect<S>, props: &mut [&mut DynProp<S>]) -> DamageInfo {
        let mut dead_info = DamageInfo {
            broken: false,
            damage: eff.value,
        };

        for prop in props {
            let alter_result = prop.alter_current_value(&eff);
            let broken = prop.alter_to_min_by(&alter_result);

            dead_info.broken = broken;
            eff.value -= alter_result.delta;
        }

        dead_info
    }

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
    use crate::combats::combat_additions::{CombatEquipArmor, CombatEquipWeapon};

    use super::*;

    struct CombatUnitData {
        strength: f64,
        belief: f64,

        health_base: f64,
        health_scale: f64,
        magicka_base: f64,
        magicka_scale: f64,

        armor_hard: f64,
        armor_soft: f64,
        armor_mass: f64,

        weapon_sharp: f64,
        weapon_mass: f64,

        shield_substitute: f64,
        shield_arcane: f64,

        health_recover_ratio: f64,
        health_recover_period: f64,
    }

    const COMBAT_UNIT_DATA: CombatUnitData = CombatUnitData {
        strength: 60.0,
        belief: 80.0,

        health_base: 20.0,
        health_scale: 1.0,
        magicka_base: 50.0,
        magicka_scale: 3.0,

        armor_hard: 12.0,
        armor_soft: 13.0,
        armor_mass: 17.0,

        weapon_sharp: 1.0,
        weapon_mass: 5.0,

        shield_substitute: 19.0,
        shield_arcane: 6.0,

        health_recover_ratio: 0.001,
        health_recover_period: 1.0,
    };

    const MAGICKA_ENERGY_LEVEL: MagickaEnergyLevel = MagickaEnergyLevel::new(100.0, 200.0, 300.0);

    fn gen_combat_inherent_attr() -> CombatInherentAttr<&'static str> {
        CombatInherentAttr::new(COMBAT_UNIT_DATA.strength, COMBAT_UNIT_DATA.belief)
    }

    fn gen_combat_addition_attr() -> CombatAdditionAttr<&'static str> {
        let mut combat_addition_attr = CombatAdditionAttr::new();
        combat_addition_attr.apply_equip_armor(
            "armor_eff",
            &CombatEquipArmor::new(
                "armor_name",
                COMBAT_UNIT_DATA.armor_hard,
                COMBAT_UNIT_DATA.armor_soft,
                COMBAT_UNIT_DATA.armor_mass,
            ),
        );
        combat_addition_attr.apply_equip_weapon(
            "weapon_eff",
            &CombatEquipWeapon::new(
                "weapon_name",
                COMBAT_UNIT_DATA.weapon_sharp,
                COMBAT_UNIT_DATA.weapon_mass,
            ),
        );
        combat_addition_attr
    }

    fn gen_combat_unit(
        combat_inherent_attr: CombatInherentAttr<&'static str>,
        combat_addition_attr: CombatAdditionAttr<&'static str>,
    ) -> CombatUnit<&'static str> {
        let mut combat_unit = CombatUnit::new(
            COMBAT_UNIT_DATA.health_base,
            COMBAT_UNIT_DATA.health_scale,
            COMBAT_UNIT_DATA.magicka_base,
            COMBAT_UNIT_DATA.magicka_scale,
            combat_inherent_attr,
            combat_addition_attr,
            &MAGICKA_ENERGY_LEVEL,
        );

        combat_unit.init_health_eff(
            "from_name",
            "health_recover_name",
            COMBAT_UNIT_DATA.health_recover_ratio,
            COMBAT_UNIT_DATA.health_recover_period,
        );

        combat_unit.init_addition_eff("from_name", "effect_name");

        // 手动添加护盾
        combat_unit.add_arcane_shield_eff(DynPropDurEffect::new_max_val(
            EffectBuilder::new_infinite("from_name", "effect_name", COMBAT_UNIT_DATA.shield_arcane),
        ));
        combat_unit.add_substitute_shield_eff(DynPropDurEffect::new_max_val(
            EffectBuilder::new_infinite(
                "from_name",
                "effect_name",
                COMBAT_UNIT_DATA.shield_substitute,
            ),
        ));

        combat_unit
    }

    #[test]
    fn combat_unit_new_and_init_addition_eff() {
        // inherent_attr
        let combat_inherent_attr = gen_combat_inherent_attr();
        assert_eq!(
            combat_inherent_attr.strength.get_current(),
            COMBAT_UNIT_DATA.strength
        );
        assert_eq!(
            combat_inherent_attr.belief.get_current(),
            COMBAT_UNIT_DATA.belief
        );

        // addition_attr
        let combat_addition_attr = gen_combat_addition_attr();
        assert_eq!(
            combat_addition_attr.armor_hard.get_current(),
            COMBAT_UNIT_DATA.armor_hard
        );
        assert_eq!(
            combat_addition_attr.armor_soft.get_current(),
            COMBAT_UNIT_DATA.armor_soft
        );
        assert_eq!(
            combat_addition_attr.armor_mass.get_current(),
            COMBAT_UNIT_DATA.armor_mass
        );
        assert_eq!(
            combat_addition_attr.weapon_sharp.get_current(),
            COMBAT_UNIT_DATA.weapon_sharp
        );
        assert_eq!(
            combat_addition_attr.weapon_mass.get_current(),
            COMBAT_UNIT_DATA.weapon_mass
        );

        // combat_unit
        let combat_unit = gen_combat_unit(combat_inherent_attr, combat_addition_attr);

        // health
        let health_value = NumericalBalancer::calc_health_max(
            COMBAT_UNIT_DATA.health_base,
            COMBAT_UNIT_DATA.health_scale,
            &combat_unit.inherent_attr,
        );
        assert_eq!(health_value, 20.0 + 1.0 * 60.0);
        assert_eq!(health_value, 80.0);
        assert_eq!(
            combat_unit.health_shields.health.get_current(),
            health_value
        );

        // shield_value
        let shield_defence_value =
            NumericalBalancer::calc_defence_shield(&combat_unit.addition_attr);
        assert_eq!(shield_defence_value, 12.0);
        assert_eq!(
            combat_unit.health_shields.shield_defence.get_current(),
            shield_defence_value
        );
        let shield_arcane_value = COMBAT_UNIT_DATA.shield_arcane;
        assert_eq!(shield_arcane_value, 6.0);
        assert_eq!(
            combat_unit.health_shields.shield_arcane.get_current(),
            shield_arcane_value
        );
        let shield_substitute_value = COMBAT_UNIT_DATA.shield_substitute;
        assert_eq!(shield_substitute_value, 19.0);
        assert_eq!(
            combat_unit.health_shields.shield_substitute.get_current(),
            shield_substitute_value
        );

        // magicka
        let magicka_value = NumericalBalancer::calc_magicka_value(
            COMBAT_UNIT_DATA.magicka_base,
            COMBAT_UNIT_DATA.magicka_scale,
            &combat_unit.inherent_attr,
        );
        assert_eq!(magicka_value, 50.0 + 3.0 * 80.0);
        assert_eq!(magicka_value, 290.0);
        let magicka_max = NumericalBalancer::calc_magicka_max(
            COMBAT_UNIT_DATA.magicka_base,
            COMBAT_UNIT_DATA.magicka_scale,
            &combat_unit.inherent_attr,
            &MAGICKA_ENERGY_LEVEL,
        );
        assert_eq!(magicka_max, MAGICKA_ENERGY_LEVEL.max_energy(magicka_value));
        assert_eq!(magicka_max, 300.0);
        assert_eq!(combat_unit.magicka.get_current(), 0.0);
        assert_eq!(combat_unit.magicka.get_max(), magicka_max);

        // stamina
        assert_eq!(
            combat_unit.stamina.get_current(),
            NumericalBalancer::get_default_prop_value()
        );
    }

    #[test]
    fn hurt_and_recover() {
        let combat_inherent_attr = gen_combat_inherent_attr();
        let combat_addition_attr = gen_combat_addition_attr();
        let mut combat_unit = gen_combat_unit(combat_inherent_attr, combat_addition_attr);

        // 确认前提条件 若有不符自行修正
        let mut health_value = 80.0;
        assert_eq!(
            health_value,
            combat_unit.health_shields.health.get_current()
        );
        let mut shield_defence_value = 12.0;
        assert_eq!(
            shield_defence_value,
            combat_unit.health_shields.shield_defence.get_current()
        );
        let mut shield_arcane_value = 6.0;
        assert_eq!(
            shield_arcane_value,
            combat_unit.health_shields.shield_arcane.get_current()
        );
        let mut shield_substitute_value = 19.0;
        assert_eq!(
            shield_substitute_value,
            combat_unit.health_shields.shield_substitute.get_current()
        );

        // hurt
        combat_unit.health_shields.hurt_external(
            DamageType::KarmaTruth,
            DynPropInstEffect::new_val(EffectBuilder::new_instant("fff", "eee", -1.0)),
            1.0,
        );
        health_value -= 1.0;
        assert_eq!(
            health_value,
            combat_unit.health_shields.health.get_current()
        );

        combat_unit.health_shields.hurt_external(
            DamageType::PhysicsImpact,
            DynPropInstEffect::new_val(EffectBuilder::new_instant("fff", "eee", -1.0)),
            1.0,
        );
        shield_substitute_value -= 1.0;
        assert_eq!(
            shield_substitute_value,
            combat_unit.health_shields.shield_substitute.get_current()
        );

        combat_unit.health_shields.hurt_external(
            DamageType::PhysicsShear,
            DynPropInstEffect::new_val(EffectBuilder::new_instant("fff", "eee", -1.0)),
            1.0,
        );
        shield_defence_value -= 1.0;
        assert_eq!(
            shield_defence_value,
            combat_unit.health_shields.shield_defence.get_current()
        );

        combat_unit.health_shields.hurt_external(
            DamageType::MagickaArcane,
            DynPropInstEffect::new_val(EffectBuilder::new_instant("fff", "eee", -1.0)),
            1.0,
        );
        shield_arcane_value -= 1.0;
        assert_eq!(
            shield_arcane_value,
            combat_unit.health_shields.shield_arcane.get_current()
        );

        assert_eq!(health_value, 79.0);
        assert_eq!(shield_substitute_value, 18.0);
        assert_eq!(shield_defence_value, 11.0);
        assert_eq!(shield_arcane_value, 5.0);

        combat_unit.health_shields.hurt_external(
            DamageType::MagickaArcane,
            DynPropInstEffect::new_val(EffectBuilder::new_instant("fff", "eee", -6.0)),
            1.0,
        );
        shield_arcane_value -= 5.0;
        shield_substitute_value -= 1.0;
        assert_eq!(health_value, 79.0);
        assert_eq!(shield_substitute_value, 17.0);
        assert_eq!(shield_defence_value, 11.0);
        assert_eq!(shield_arcane_value, 0.0);
        assert_eq!(
            health_value,
            combat_unit.health_shields.health.get_current()
        );
        assert_eq!(
            shield_substitute_value,
            combat_unit.health_shields.shield_substitute.get_current()
        );
        assert_eq!(
            shield_defence_value,
            combat_unit.health_shields.shield_defence.get_current()
        );
        assert_eq!(
            shield_arcane_value,
            combat_unit.health_shields.shield_arcane.get_current()
        );

        combat_unit.health_shields.hurt_external(
            DamageType::PhysicsImpact,
            DynPropInstEffect::new_val(EffectBuilder::new_instant("fff", "eee", -20.0)),
            1.0,
        );
        shield_substitute_value -= 17.0;
        health_value -= 3.0;
        assert_eq!(health_value, 76.0);
        assert_eq!(shield_substitute_value, 0.0);
        assert_eq!(shield_defence_value, 11.0);
        assert_eq!(shield_arcane_value, 0.0);
        assert_eq!(
            health_value,
            combat_unit.health_shields.health.get_current()
        );
        assert_eq!(
            shield_substitute_value,
            combat_unit.health_shields.shield_substitute.get_current()
        );
        assert_eq!(
            shield_defence_value,
            combat_unit.health_shields.shield_defence.get_current()
        );
        assert_eq!(
            shield_arcane_value,
            combat_unit.health_shields.shield_arcane.get_current()
        );

        combat_unit.health_shields.hurt_external(
            DamageType::PhysicsShear,
            DynPropInstEffect::new_val(EffectBuilder::new_instant("fff", "eee", -15.0)),
            1.0,
        );
        shield_defence_value -= 11.0;
        health_value -= 4.0;
        assert_eq!(health_value, 72.0);
        assert_eq!(shield_substitute_value, 0.0);
        assert_eq!(shield_defence_value, 0.0);
        assert_eq!(shield_arcane_value, 0.0);
        assert_eq!(
            health_value,
            combat_unit.health_shields.health.get_current()
        );
        assert_eq!(
            shield_substitute_value,
            combat_unit.health_shields.shield_substitute.get_current()
        );
        assert_eq!(
            shield_defence_value,
            combat_unit.health_shields.shield_defence.get_current()
        );
        assert_eq!(
            shield_arcane_value,
            combat_unit.health_shields.shield_arcane.get_current()
        );

        // recover
        // 每秒恢复 0.8
        assert_eq!(combat_unit.health_shields.health.get_max(), 80.0);
        assert_eq!(COMBAT_UNIT_DATA.health_recover_ratio, 0.001);
        assert_eq!(COMBAT_UNIT_DATA.health_recover_period, 1.0);
        let mut the_time = 0.0;
        let mut the_delta = 0.0;
        assert_eq!(the_delta, 0.0);

        the_delta = 0.5;
        the_time += the_delta;
        assert_eq!(the_time, 0.5);
        combat_unit.process_time(the_delta);
        assert_eq!(combat_unit.health_shields.health.get_current(), 72.0);

        the_delta = 0.6;
        the_time += the_delta;
        assert_eq!(the_time, 1.1);
        combat_unit.process_time(the_delta);
        assert_eq!(combat_unit.health_shields.health.get_current(), 72.08);

        the_delta = 0.9;
        the_time += the_delta;
        assert_eq!(the_time, 2.0);
        combat_unit.process_time(the_delta);
        assert_eq!(combat_unit.health_shields.health.get_current(), 72.16);

        the_delta = 2.3;
        the_time += the_delta;
        assert_eq!(the_time, 4.3);
        combat_unit.process_time(the_delta);
        assert_eq!(combat_unit.health_shields.health.get_current(), 72.32);

        the_delta = 5.4;
        the_time += the_delta;
        assert_eq!(the_time, 9.7);
        combat_unit.process_time(the_delta);
        assert_eq!(combat_unit.health_shields.health.get_current(), 72.72);
    }
}

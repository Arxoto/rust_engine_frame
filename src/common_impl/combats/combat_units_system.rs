//! 战斗单位系统
//!
//! 设计约束:
//! - 系统函数不含数值公式/魔法数字(除 [`gen_shield_defence`] 固化的防护盾公式)。
//! - 奥术/替身护盾只做装载,数值公式是给数值策划的要求,由调用方算好值传入。
//! - 减益/伤害类入参用负值(与 `EffectMeaning::Bad` 语义一致)。
//! - 涉及效果的**持久**封装入参传完整 [`crate::base_lib::eff_attr_prop::effects::Effect`];
//!   即时(非持久)资源变更(如扣能量/削韧)收裸数值,无 id 参与 upsert 故不适用。

use crate::{
    base_lib::{
        cores::{timers::static_timer::StaticTimer, unify_types::FixedName},
        eff_attr_prop::{
            effects::Effect,
            prop_bounds_eff::{PropBoundsEffect, PropBoundsEffectType},
            props::{Prop, PropAlterResult},
            upsert_container::UpsertContainer,
        },
    },
    common_impl::combats::{
        combat_additions::ArmorHard,
        combat_inherents::{Belief, Strength},
        combat_units::{Health, Magicka, Stamina},
        damages::{MagickaEnergyLevel, damage_system},
    },
};

/// 花费能量后允许的最低值(资源下限门槛,即时扣减用)
const COST_FLOOR: f64 = 0.0;

/// 初始化三维的配置参数(由上层传入,不含魔法数字)
pub struct ThreeBarsConfig {
    pub health_base: f64,
    pub health_scale: f64,
    pub magicka_base: f64,
    pub magicka_scale: f64,
    pub stamina_max: f64,
    pub magicka_energy_level: MagickaEnergyLevel,
}

/// 初始化三维:按内禀属性设置血量/平衡/能量的最大值与当前值
///
/// 由上层在角色创建时调用一次,使角色从出生起即可战斗。
/// - 血量:`current = max`,经 [`damage_system::calc_health_max`]
/// - 平衡:`current = max = config.stamina_max`(与任何内禀属性无关,所有角色相等)
/// - 能量:`current = 0`、`max` 经 [`damage_system::calc_magicka_max`]
pub fn init_three_bars(
    health: &mut Health,
    stamina: &mut Stamina,
    magicka: &mut Magicka,
    strength: &Strength,
    belief: &Belief,
    config: &ThreeBarsConfig,
) {
    let health_max =
        damage_system::calc_health_max(config.health_base, config.health_scale, strength);
    let magicka_max = damage_system::calc_magicka_max(
        config.magicka_base,
        config.magicka_scale,
        belief,
        &config.magicka_energy_level,
    );

    *health = Health(Prop::new(health_max, health_max, 0.0));
    *stamina = Stamina(Prop::new(config.stamina_max, config.stamina_max, 0.0));
    *magicka = Magicka(Prop::new(0.0, magicka_max, 0.0));
}

/// 根据外赋属性生成防护护盾值
///
/// 固化公式 [`damage_system::calc_defence_shield`]:防护护盾值 = 盔甲坚韧当前值。
///
/// **只生成,不装载**:返回值是护盾值,由调用方构造 [`Effect`] 后
/// 手动调用装载方法(见 [`load_shield`])装载。
pub fn gen_shield_defence(armor_hard: &ArmorHard) -> f64 {
    damage_system::calc_defence_shield(armor_hard)
}

/// 装载护盾:将护盾值写入护盾,同时影响最大值与当前值
///
/// 对防护/奥术/替身护盾通用:调用方传 `&mut shield.0` 与 `&mut shield_effs.0`。
///
/// 效果驱动机制(见 `prop_bounds_eff.rs` 文档「若想同时修改上限与实际值」):
/// 1. 用传入 [`Effect`] 构造 `UpperAdd` 边界效果并 upsert 进容器——id 由
///    `Effect` 的 from/eff 名决定,重复装载同 id 幂等覆盖。
/// 2. [`Prop::refresh_bounds`] 重算上限。
/// 3. [`Prop::apply_eff`] 提升当前值至装载值。
///
/// 数值由调用方传入(奥术/替身护盾的公式是给数值策划的要求),本函数不含公式。
pub fn load_shield<S: FixedName>(
    shield_effs: &mut UpsertContainer<PropBoundsEffect<S, StaticTimer>>,
    shield: &mut Prop,
    eff: Effect<S>,
) {
    let value = eff.get_effect_value();
    let bounds_eff = PropBoundsEffect::new(PropBoundsEffectType::UpperAdd, eff, StaticTimer::inf());
    shield_effs.upsert_replace(bounds_eff);

    shield.refresh_bounds(shield_effs.iter_ele());
    shield.apply_eff(value);
}

/// 花费能量(硬扣):不检查是否足够,直接扣减,返回实际生效值
///
/// 用于系统级强制扣除(如死亡惩罚)。入参为负值(花费 = 减益,
/// 与 `EffectMean::Bad` 语义一致)。被上下限钳制时,`real_eff_val` 反映实际生效量。
pub fn cost_magicka(magicka: &mut Magicka, eff_val: f64) -> PropAlterResult {
    magicka.0.apply_eff(eff_val)
}

/// 尝试花费能量(软扣):能量不足则失败,返回 `None` 且值不变
///
/// 用于施法前置检查(魔法不够则施放失败)。门槛为花费后不低于 0。
pub fn try_cost_magicka(magicka: &mut Magicka, eff_val: f64) -> Option<PropAlterResult> {
    magicka.0.apply_eff_checked(eff_val, COST_FLOOR)
}

/// 削韧:削减平衡,返回实际生效值
///
/// 入参为负值(削韧 = 减益,与 `EffectMean::Bad` 语义一致)。
/// 清空时(平衡降至下限 0)触发倒地,由上层通过返回值或 [`Prop::current_is_zero`] 判断。
pub fn cut_stamina(stamina: &mut Stamina, eff_val: f64) -> PropAlterResult {
    stamina.0.apply_eff(eff_val)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base_lib::eff_attr_prop::attrs::Attr, common_impl::combats::combat_units::ShieldDefence,
    };

    /// 防护护盾值 = 盔甲坚韧当前值(公式固化,独立字面量断言)
    #[test]
    fn gen_shield_defence_returns_armor_hard_value() {
        let armor_hard = ArmorHard(Attr::new(80.0));
        assert_eq!(gen_shield_defence(&armor_hard), 80.0);
    }

    /// 装载护盾:上限与当前值同时变为装载值
    #[test]
    fn load_shield_sets_max_and_current() {
        let mut shield_effs: UpsertContainer<PropBoundsEffect<String, StaticTimer>> =
            UpsertContainer::default();
        let mut shield = ShieldDefence(Prop::new(0.0, 0.0, 0.0));

        load_shield(
            &mut shield_effs,
            &mut shield.0,
            Effect::new("player".to_string(), "defence_shield".to_string(), 80.0),
        );

        assert_eq!(shield.0.get_max(), 80.0);
        assert_eq!(shield.0.get_current(), 80.0);
    }

    /// 装载护盾:同 id 重复装载幂等覆盖(新值替换旧值,容器不新增)
    #[test]
    fn load_shield_same_id_replaces() {
        let mut shield_effs: UpsertContainer<PropBoundsEffect<String, StaticTimer>> =
            UpsertContainer::default();
        let mut shield = ShieldDefence(Prop::new(0.0, 0.0, 0.0));

        load_shield(
            &mut shield_effs,
            &mut shield.0,
            Effect::new("player".to_string(), "defence_shield".to_string(), 80.0),
        );
        load_shield(
            &mut shield_effs,
            &mut shield.0,
            Effect::new("player".to_string(), "defence_shield".to_string(), 120.0),
        );

        assert_eq!(shield.0.get_max(), 120.0);
        assert_eq!(shield.0.get_current(), 120.0);
        assert_eq!(shield_effs.ele_len(), 1); // 同 id 不新增
    }

    /// 硬扣能量:负数扣减,超出上限(此处为下限 0)时被钳制,返回实际生效值
    #[test]
    fn cost_magicka_deducts_and_clamps() {
        let mut magicka = Magicka(Prop::new(30.0, 100.0, 0.0));

        // 正常扣减:30 - 20 = 10
        let res = cost_magicka(&mut magicka, -20.0);
        assert_eq!(magicka.0.get_current(), 10.0);
        assert_eq!(res.real_eff_val, -20.0);

        // 超限扣减:10 - 200 被钳制到下限 0,实际只生效 10
        let res = cost_magicka(&mut magicka, -200.0);
        assert_eq!(magicka.0.get_current(), 0.0);
        assert_eq!(res.real_eff_val, -10.0);
    }

    /// 软扣能量:充足时扣减并返回实际生效值,不足时返回 None 且值不变
    #[test]
    fn try_cost_magicka_gates_on_sufficient() {
        let mut magicka = Magicka(Prop::new(30.0, 100.0, 0.0));

        // 充足:30 - 20 = 10
        let res = try_cost_magicka(&mut magicka, -20.0).expect("能量充足应扣减");
        assert_eq!(magicka.0.get_current(), 10.0);
        assert_eq!(res.real_eff_val, -20.0);

        // 不足:10 - 40 < 0 → None,值不变
        assert!(try_cost_magicka(&mut magicka, -40.0).is_none());
        assert_eq!(magicka.0.get_current(), 10.0);
    }

    /// 削韧:负数入参削减平衡,清空触发倒地(下限 0),返回实际生效值
    #[test]
    fn cut_stamina_deducts_balance() {
        let mut stamina = Stamina(Prop::new(100.0, 100.0, 0.0));

        // 正常削韧:100 - 20 = 80
        let res = cut_stamina(&mut stamina, -20.0);
        assert_eq!(stamina.0.get_current(), 80.0);
        assert_eq!(res.real_eff_val, -20.0);

        // 超限削韧:80 - 100 被钳制到下限 0(清空→倒地),实际只生效 80
        let res = cut_stamina(&mut stamina, -100.0);
        assert_eq!(stamina.0.get_current(), 0.0);
        assert_eq!(res.real_eff_val, -80.0);
        assert!(stamina.0.current_is_zero());
    }

    /// 初始化三维:血量/平衡满,能量为零,上限按内禀与配置计算
    #[test]
    fn init_three_bars_sets_bars_from_inherents() {
        let mut health = Health(Prop::default());
        let mut stamina = Stamina(Prop::default());
        let mut magicka = Magicka(Prop::default());
        let strength = Strength(Attr::new(5.0));
        let belief = Belief(Attr::new(5.0));
        let energy_level = MagickaEnergyLevel::new(100.0, 200.0, 300.0);

        let config = ThreeBarsConfig {
            health_base: 100.0,
            health_scale: 10.0,
            magicka_base: 50.0,
            magicka_scale: 20.0,
            stamina_max: 100.0,
            magicka_energy_level: energy_level,
        };
        init_three_bars(
            &mut health,
            &mut stamina,
            &mut magicka,
            &strength,
            &belief,
            &config,
        );

        // 血量 max = 100 + 10*5 = 150,current 满
        assert_eq!(health.0.get_max(), 150.0);
        assert_eq!(health.0.get_current(), 150.0);
        // 平衡 max = current = stamina_max
        assert_eq!(stamina.0.get_max(), 100.0);
        assert_eq!(stamina.0.get_current(), 100.0);
        // 能量 原始值 = 50 + 20*5 = 150 → 能级映射到 200,current 从零开始
        assert_eq!(magicka.0.get_max(), 200.0);
        assert_eq!(magicka.0.get_current(), 0.0);
    }
}

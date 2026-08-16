//! 战斗单位系统
//!
//! 设计约束:
//! - 系统函数不含数值公式/魔法数字(除 [`gen_shield_defence`] 固化的防护盾公式)。
//! - 奥术/替身护盾只做装载,数值公式是给数值策划的要求,由调用方算好值传入。
//! - 减益/伤害类入参用负值(与 `EffectMeaning::Bad` 语义一致)。
//! - 涉及效果的上层封装传完整 [`crate::base_lib::eff_attr_prop::effects::Effect`]:
//!   持久效果(护盾上限)直接传 `Effect`;即时变更(扣能量/削韧)经
//!   [`crate::base_lib::eff_attr_prop::prop_alter_eff::PropAlterEffect`] 包装传完整 `Effect`。
//! - ECS 语义:护盾上限由 [`crate::base_lib::eff_attr_prop::prop_systems`] 每帧依脏标签刷新,
//!   护盾当前值由伤害管线经缓冲消费([`load_shield`] 仅编排不入值);
//!   即时变更(cost/cut)直接应用,不经缓冲。

use crate::{
    base_lib::{
        cores::{timers::static_timer::StaticTimer, unify_types::FixedName},
        eff_attr_prop::{
            effects::Effect,
            prop_alter_eff::{PropAlterEffect, alter_abs_value, apply_prop_alter_eff},
            prop_bounds_eff::{PropBoundsEffect, PropBoundsEffectType},
            props::{Prop, PropAlterResult},
            upsert_container::UpsertContainer,
        },
    },
    common_impl::combats::{
        combat_additions::ArmorHard,
        combat_inherents::{Belief, Strength},
        combat_units::{Health, Magicka, Stamina},
        damages::{DamageEffect, DamageEffectBuffer, MagickaEnergyLevel, damage_system},
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

/// 角色出生三维(按内禀属性生成,返回所有权装配到实体)
pub struct ThreeBars {
    pub health: Health,
    pub stamina: Stamina,
    pub magicka: Magicka,
}

/// 生成三维:按内禀属性返回血量/平衡/能量的所有权
///
/// 由上层在角色创建时调用一次,将结果装配到实体组件。
/// - 血量:`current = max`,经 [`damage_system::calc_health_max`]
/// - 平衡:`current = max = config.stamina_max`(与任何内禀属性无关,所有角色相等)
/// - 能量:`current = 0`、`max` 经 [`damage_system::calc_magicka_max`]
pub fn gen_three_bars(strength: &Strength, belief: &Belief, config: &ThreeBarsConfig) -> ThreeBars {
    let health_max =
        damage_system::calc_health_max(config.health_base, config.health_scale, strength);
    let magicka_max = damage_system::calc_magicka_max(
        config.magicka_base,
        config.magicka_scale,
        belief,
        &config.magicka_energy_level,
    );

    ThreeBars {
        health: Health(Prop::new(health_max, health_max, 0.0)),
        stamina: Stamina(Prop::new(config.stamina_max, config.stamina_max, 0.0)),
        magicka: Magicka(Prop::new(0.0, magicka_max, 0.0)),
    }
}

/// 根据外赋属性生成防护护盾的**上限效果**
///
/// 固化公式 [`damage_system::calc_defence_shield`]:防护护盾值 = 盔甲坚韧当前值。
/// 返回就绪的 `UpperAdd` [`PropBoundsEffect`],由调用方 upsert 进护盾的 `*Effs` 容器,
/// 上限刷新交给 [`crate::base_lib::eff_attr_prop::prop_systems::try_refresh_dirty_prop_bounds`]。
///
/// 护盾**当前值**的装载由调用方另行构造 `OnlyShieldDefence` 的 [`DamageEffect`] 推入缓冲。
pub fn gen_shield_defence<S: FixedName>(
    armor_hard: &ArmorHard,
    from_name: S,
    effect_name: S,
) -> PropBoundsEffect<S, StaticTimer> {
    let value = damage_system::calc_defence_shield(armor_hard);
    PropBoundsEffect::new(
        PropBoundsEffectType::UpperAdd,
        Effect::new(from_name, effect_name, value),
        StaticTimer::inf(),
    )
}

/// 装载护盾:编排"上限效果入容器 + 当前值效果入缓冲"
///
/// **不做同步修改**(符合 ECS):上限由每帧 [`crate::base_lib::eff_attr_prop::prop_systems`]
/// 依脏标签刷新,当前值由伤害管线消费。调用方:
/// - `bounds_eff` 来自 [`gen_shield_defence`] 或调用方自建;
/// - `value_eff` 为 `Only*` 类型的 [`DamageEffect`](如 `OnlyShieldDefence`),正值累加护盾当前值。
pub fn load_shield<S: FixedName>(
    shield_effs: &mut UpsertContainer<PropBoundsEffect<S, StaticTimer>>,
    damage_buffer: &mut DamageEffectBuffer<S>,
    bounds_eff: PropBoundsEffect<S, StaticTimer>,
    value_eff: DamageEffect<S>,
) {
    shield_effs.upsert_replace(bounds_eff);
    damage_buffer.push(value_eff);
}

/// 花费能量(硬扣):直接修改,返回实际生效值
///
/// 用于系统级强制扣除(如死亡惩罚)。入参为负值(花费 = 减益,
/// 与 `EffectMean::Bad` 语义一致)。被上下限钳制时,`real_eff_val` 反映实际生效量。
pub fn cost_magicka<S: FixedName>(
    magicka: &mut Magicka,
    eff: PropAlterEffect<S>,
) -> PropAlterResult {
    apply_prop_alter_eff(&mut magicka.0, eff)
}

/// 尝试花费能量(软扣):能量不足则失败,返回 `None` 且值不变
///
/// 用于施法前置检查(魔法不够则施放失败)。门槛为花费后不低于 0。
/// 按 [`crate::base_lib::eff_attr_prop::prop_alter_eff::PropAlterEffectType`] 折算后,若折算值不足则返回 `None`。
pub fn try_cost_magicka<S: FixedName>(
    magicka: &mut Magicka,
    eff: PropAlterEffect<S>,
) -> Option<PropAlterResult> {
    // 折算出绝对值(参照目标 Prop 自身),软扣判断是否可支付
    let abs_val = alter_abs_value(&magicka.0, &eff);
    magicka.0.apply_eff_checked(abs_val, COST_FLOOR)
}

/// 削韧:削减平衡,返回实际生效值
///
/// 入参为负值(削韧 = 减益,与 `EffectMean::Bad` 语义一致)。
/// 清空时(平衡降至下限 0)触发倒地,由上层通过返回值或 [`Prop::current_is_zero`] 判断。
pub fn cut_stamina<S: FixedName>(
    stamina: &mut Stamina,
    eff: PropAlterEffect<S>,
) -> PropAlterResult {
    apply_prop_alter_eff(&mut stamina.0, eff)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base_lib::{
            cores::timers::static_timer::StaticTimer,
            eff_attr_prop::{attrs::Attr, prop_alter_eff::PropAlterEffectType, props::Prop},
        },
        common_impl::combats::{combat_units::ShieldDefence, damages::DamageType},
    };

    /// 生成三维:返回血量/平衡满、能量为零的所有权
    #[test]
    fn gen_three_bars_returns_owned_bars() {
        let strength = Strength(Attr::new(5.0));
        let belief = Belief(Attr::new(5.0));
        let config = ThreeBarsConfig {
            health_base: 100.0,
            health_scale: 10.0,
            magicka_base: 50.0,
            magicka_scale: 20.0,
            stamina_max: 100.0,
            magicka_energy_level: MagickaEnergyLevel::new(100.0, 200.0, 300.0),
        };

        let bars = gen_three_bars(&strength, &belief, &config);

        // 血量 max = 100 + 10*5 = 150,current 满
        assert_eq!(bars.health.0.get_max(), 150.0);
        assert_eq!(bars.health.0.get_current(), 150.0);
        // 平衡 max = current = stamina_max
        assert_eq!(bars.stamina.0.get_max(), 100.0);
        assert_eq!(bars.stamina.0.get_current(), 100.0);
        // 能量 原始值 = 50 + 20*5 = 150 → 能级映射到 200,current 从零开始
        assert_eq!(bars.magicka.0.get_max(), 200.0);
        assert_eq!(bars.magicka.0.get_current(), 0.0);
    }

    /// gen_shield_defence:返回就绪的 UpperAdd 上限效果,经 prop_systems 刷新后上限=护盾值
    #[test]
    fn gen_shield_defence_produces_bounds_effect() {
        use crate::base_lib::eff_attr_prop::prop_systems::try_refresh_dirty_prop_bounds;

        let armor_hard = ArmorHard(Attr::new(80.0));
        let bounds_eff: PropBoundsEffect<String, StaticTimer> = gen_shield_defence(
            &armor_hard,
            "player".to_string(),
            "defence_shield".to_string(),
        );

        // upsert 上限效果入容器,由 system 依脏标签刷新
        let mut shield_effs: UpsertContainer<PropBoundsEffect<String, StaticTimer>> =
            UpsertContainer::default();
        let mut shield = ShieldDefence(Prop::new(0.0, 0.0, 0.0));
        shield_effs.upsert_replace(bounds_eff);
        try_refresh_dirty_prop_bounds(&mut shield.0, &mut shield_effs);

        // 上限 = 盔甲坚韧当前值(公式固化)
        assert_eq!(shield.0.get_max(), 80.0);
    }

    /// load_shield:只编排(上限效果入容器 + 当前值效果入缓冲),不做同步修改
    ///
    /// 断言:护盾 Prop 未被直接修改,上限由 prop_systems 刷新,当前值由伤害管线消费。
    #[test]
    fn load_shield_does_not_mutate_prop_directly() {
        let mut shield_effs: UpsertContainer<PropBoundsEffect<String, StaticTimer>> =
            UpsertContainer::default();
        let mut damage_buffer = DamageEffectBuffer::new();
        let shield = ShieldDefence(Prop::new(0.0, 0.0, 0.0));

        let bounds_eff = gen_shield_defence(
            &ArmorHard(Attr::new(80.0)),
            "player".to_string(),
            "defence_shield".to_string(),
        );
        let value_eff = DamageEffect::new(
            DamageType::OnlyShieldDefence,
            PropAlterEffectType::Val,
            Effect::new("player".to_string(), "shield_spell".to_string(), 80.0),
        );

        load_shield(&mut shield_effs, &mut damage_buffer, bounds_eff, value_eff);

        // 上限效果已入容器,脏标签置起
        assert_eq!(shield_effs.ele_len(), 1);
        assert!(shield_effs.is_changed());
        // 当前值效果已入缓冲
        assert_eq!(damage_buffer.len(), 1);
        // 护盾 Prop 未被直接修改(ECS:由 system 刷新)
        assert_eq!(shield.0.get_max(), 0.0);
        assert_eq!(shield.0.get_current(), 0.0);
    }

    /// 硬扣能量:负数扣减,超出上限(此处为下限 0)时被钳制,返回实际生效值
    #[test]
    fn cost_magicka_deducts_and_clamps() {
        let mut magicka = Magicka(Prop::new(30.0, 100.0, 0.0));

        // 正常扣减:30 - 20 = 10
        let res = cost_magicka(
            &mut magicka,
            PropAlterEffect::new(
                PropAlterEffectType::Val,
                Effect::new("from".to_string(), "cost".to_string(), -20.0),
            ),
        );
        assert_eq!(magicka.0.get_current(), 10.0);
        assert_eq!(res.real_eff_val, -20.0);
    }

    /// 软扣能量:充足时扣减并返回实际生效值,不足时返回 None 且值不变
    #[test]
    fn try_cost_magicka_gates_on_sufficient() {
        let mut magicka = Magicka(Prop::new(30.0, 100.0, 0.0));

        // 充足:30 - 20 = 10
        let res = try_cost_magicka(
            &mut magicka,
            PropAlterEffect::new(
                PropAlterEffectType::Val,
                Effect::new("from".to_string(), "cost".to_string(), -20.0),
            ),
        )
        .expect("能量充足应扣减");
        assert_eq!(magicka.0.get_current(), 10.0);
        assert_eq!(res.real_eff_val, -20.0);

        // 不足:10 - 40 < 0 → None,值不变
        assert!(
            try_cost_magicka(
                &mut magicka,
                PropAlterEffect::new(
                    PropAlterEffectType::Val,
                    Effect::new("from".to_string(), "cost".to_string(), -40.0),
                )
            )
            .is_none()
        );
        assert_eq!(magicka.0.get_current(), 10.0);
    }

    /// 软扣能量:CurPer 折算后仍按门槛判断
    #[test]
    fn try_cost_magicka_cur_per_gates() {
        let mut magicka = Magicka(Prop::new(30.0, 100.0, 0.0));
        // CurPer:-0.5 * 30 = -15,30-15=15 >= 0 → 成功
        let res = try_cost_magicka(
            &mut magicka,
            PropAlterEffect::new(
                PropAlterEffectType::CurPer,
                Effect::new("from".to_string(), "cost".to_string(), -0.5),
            ),
        )
        .expect("当前值百分比应可支付");
        assert_eq!(magicka.0.get_current(), 15.0);
        assert_eq!(res.real_eff_val, -15.0);
    }

    /// 削韧:负数入参削减平衡,清空触发倒地(下限 0),返回实际生效值
    #[test]
    fn cut_stamina_deducts_balance() {
        let mut stamina = Stamina(Prop::new(100.0, 100.0, 0.0));

        // 正常削韧:100 - 20 = 80
        let res = cut_stamina(
            &mut stamina,
            PropAlterEffect::new(
                PropAlterEffectType::Val,
                Effect::new("from".to_string(), "cut".to_string(), -20.0),
            ),
        );
        assert_eq!(stamina.0.get_current(), 80.0);
        assert_eq!(res.real_eff_val, -20.0);

        // 超限削韧:80 - 100 被钳制到下限 0(清空→倒地),实际只生效 80
        let res = cut_stamina(
            &mut stamina,
            PropAlterEffect::new(
                PropAlterEffectType::Val,
                Effect::new("from".to_string(), "cut".to_string(), -100.0),
            ),
        );
        assert_eq!(stamina.0.get_current(), 0.0);
        assert_eq!(res.real_eff_val, -80.0);
        assert!(stamina.0.current_is_zero());
    }
}

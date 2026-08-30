//! 战斗单位系统
//!
//! 设计约束:
//! - 护盾只做装载,数值公式是给数值策划的要求,不在代码里体现（初始化生成的防护护盾除外）。
//! - 减益伤害类入参用负值(与 `EffectMeaning::Bad` 语义一致)。
//! - ECS 语义:护盾上限由 [`crate::base_lib::eff_attr::attr_systems`] 每帧依脏标签刷新,
//!   护盾当前值由伤害管线经缓冲消费([`load_shield_or_health_upper`] 仅编排不入值);
//!   即时变更(cost/cut)直接应用,不经缓冲。

use slotmap::DefaultKey;

use crate::{
    base_lib::{
        cores::unify_types::FixedName,
        eff_attr::{
            bound_attr_modifiers::{BoundAttrModifier, BoundAttrModifyDimension},
            bound_attrs::BoundAttr,
            bounded_attr_modifiers::AttrAlterEff,
            bounded_attrs::BoundedAttr,
            effects::Effect,
            modifier_collections::ModifierCollection,
        },
    },
    common_impl::combats::{
        combat_additions::ArmorHard,
        combat_inherents::{Belief, Strength},
        combat_units::{
            Health, HealthLower, HealthUpper, Magicka, MagickaUpper, Stamina, StaminaUpper,
        },
        damages::{SurvivalAttrEff, SurvivalEffBuffer, SurvivalEffTargets, damage_system},
        energies::{EnergyEffBuffer, MagickaEnergyLevel},
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
    pub health_upper: HealthUpper,
    pub health_lower: HealthLower,
    pub stamina_upper: StaminaUpper,
    pub magicka_upper: MagickaUpper,
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
        health: Health(BoundedAttr::new(health_max)),
        stamina: Stamina(BoundedAttr::new(config.stamina_max)),
        magicka: Magicka(BoundedAttr::new(0.0)),
        health_upper: HealthUpper(BoundAttr::new(health_max)),
        health_lower: HealthLower(BoundAttr::new(0.0)),
        stamina_upper: StaminaUpper(BoundAttr::new(config.stamina_max)),
        magicka_upper: MagickaUpper(BoundAttr::new(magicka_max)),
    }
}

/// 根据外赋属性计算并生成初始的防护护盾值
pub fn gen_shield_defence<S: FixedName>(
    armor_hard: &ArmorHard,
    from_name: S,
    effect_name: S,
) -> (BoundAttrModifier, SurvivalAttrEff<S>) {
    let shield_defence_val = damage_system::calc_defence_shield(armor_hard);
    let effect = Effect::new(from_name, effect_name, shield_defence_val);

    let (bound_eff, alter_eff) = AttrAlterEff::gen_effs_for_upper_bound_by_val(effect);
    let svv_eff =
        SurvivalAttrEff::new_from_alter_eff(SurvivalEffTargets::OnlyShieldDefence, alter_eff);

    (bound_eff, svv_eff)
}

/// 生成通用护盾，也可用于提升生命值最大值
pub fn gen_shield_or_health_upper<S: FixedName>(
    upper_bound: &BoundAttr,
    eff_type: BoundAttrModifyDimension,
    effect: Effect<S>,
) -> (BoundAttrModifier, SurvivalAttrEff<S>) {
    let (bound_eff, alter_eff) =
        AttrAlterEff::gen_effs_for_upper_bound(upper_bound, eff_type, effect);
    let svv_eff =
        SurvivalAttrEff::new_from_alter_eff(SurvivalEffTargets::OnlyShieldDefence, alter_eff);

    (bound_eff, svv_eff)
}

/// 装载护盾: 编排"上限效果入容器 + 当前值效果入缓冲"
///
/// **不做同步修改** 由 system 驱动更新
pub fn load_shield_or_health_upper<S: FixedName>(
    shield_effs: &mut ModifierCollection<BoundAttrModifier>,
    svv_eff_buffer: &mut SurvivalEffBuffer<S>,
    bounds_eff: BoundAttrModifier,
    value_eff: SurvivalAttrEff<S>,
) -> DefaultKey {
    svv_eff_buffer.push(value_eff);
    shield_effs.insert(bounds_eff)
}

/// 花费能量(硬扣): 直接修改, 推入 buffer
pub fn cost_magicka<S: FixedName>(buffer: &mut EnergyEffBuffer<S>, eff: AttrAlterEff<S>) {
    buffer.push(eff);
}

/// 尝试花费能量(软扣): 能量不足则失败, 顺序必须在能量系统消费 buffer 之后
pub fn try_cost_magicka<S: FixedName>(
    magicka: &mut Magicka,
    magicka_upper: &MagickaUpper,
    eff: AttrAlterEff<S>,
) -> bool {
    let bounded_attr = &mut magicka.0;
    let abs_val = eff.calc_alter_val(bounded_attr, &magicka_upper.0);
    bounded_attr.apply_eff_checked(COST_FLOOR, abs_val, COST_FLOOR)
}

/// 削韧: 削减平衡, 返回实际生效值
///
/// 由于设计比较简单，无需通过 buffer ，直接生效
pub fn cut_stamina<S: FixedName>(
    stamina: &mut Stamina,
    stamina_upper: &StaminaUpper,
    eff: AttrAlterEff<S>,
) {
    let bounded_attr = &mut stamina.0;
    let abs_val = eff.calc_alter_val(bounded_attr, &stamina_upper.0);
    bounded_attr.apply_eff(abs_val)
}

// todo 增加复合属性的显示函数

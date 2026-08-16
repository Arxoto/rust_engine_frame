use crate::{
    base_lib::{
        cores::unify_types::FixedName,
        eff_attr_prop::{effects::Effect, props::Prop},
    },
    common_impl::combats::{
        combat_additions::{ArmorHard, ArmorSoft, WeaponMass, WeaponSharp},
        combat_inherents::{Belief, Strength},
        combat_units::{Health, Magicka, ShieldArcane, ShieldDefence, ShieldSubstitute},
    },
};

#[derive(Debug)]
pub struct DamageEffectBuffer<S: FixedName>(Vec<DamageEffect<S>>);

/// 伤害信息，表示每次伤害造成的影响
#[derive(Debug)]
pub struct DamageInfo<S: FixedName> {
    /// 首次造成伤害的来源和效果名称，用于统计死因
    pub first_hurt_heal_from_eff: Option<(S, S)>,
}

#[derive(Debug, Clone)]
pub struct DamageEffect<S: FixedName> {
    dmg_type: DamageType,
    dmg_calc: DamageCalc,
    eff: Effect<S>,
}

impl<S: FixedName> DamageEffect<S> {
    /// 构造一次伤害效果
    ///
    /// 公开构造路径：上层无需访问私有字段即可创建伤害输入，
    /// 推入 [`DamageEffectBuffer`] 后由伤害系统消费。
    #[must_use]
    pub fn new(dmg_type: DamageType, dmg_calc: DamageCalc, eff: Effect<S>) -> Self {
        Self {
            dmg_type,
            dmg_calc,
            eff,
        }
    }
}

impl<S: FixedName> Default for DamageEffectBuffer<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: FixedName> DamageEffectBuffer<S> {
    /// 构造空的伤害缓冲
    #[must_use]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// 推入一次伤害效果
    pub fn push(&mut self, dmg_eff: DamageEffect<S>) {
        self.0.push(dmg_eff);
    }

    /// 缓冲内伤害效果数量
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// 缓冲是否为空
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DamageType {
    /// 因果论的真实伤害
    KarmaTruth,
    /// 物理剪切 尖锐
    PhysicsShear,
    /// 物理冲击 沉重
    PhysicsImpact,
    /// 魔法奥术
    MagickaArcane,

    /// 防护破盾
    BrokeShieldDefence,
    /// 奥术破盾
    BrokeShieldArcane,
}

#[derive(Debug, Clone, Copy)]
pub enum DamageCalc {
    /// 绝对值修改
    Val,
    /// 根据当前值的百分比
    CurPer,
    /// 根据最大值的百分比
    MaxPer,
}

impl<S: FixedName> Default for DamageInfo<S> {
    fn default() -> Self {
        Self {
            first_hurt_heal_from_eff: None,
        }
    }
}

impl DamageType {
    /// 能否对血量造成伤害（剔除破盾类型）
    pub fn is_hurt_heal(&self) -> bool {
        match self {
            DamageType::KarmaTruth => true,
            DamageType::PhysicsShear => true,
            DamageType::PhysicsImpact => true,
            DamageType::MagickaArcane => true,
            DamageType::BrokeShieldDefence => false,
            DamageType::BrokeShieldArcane => false,
        }
    }
}

pub struct MagickaEnergyLevel(f64, f64, f64);

impl MagickaEnergyLevel {
    pub const fn new(l0: f64, l1: f64, l2: f64) -> MagickaEnergyLevel {
        MagickaEnergyLevel(l0, l1, l2)
    }

    pub fn max_energy(&self, v: f64) -> f64 {
        if v <= self.0 {
            self.0
        } else if v <= self.1 {
            self.1
        } else {
            self.2
        }
    }
}

/// ## 如何衡量伤害公式是否平衡
///
/// - 随着角色成长，【伤害成长】应该与【受伤上限】大致成正比
/// - 伤害公式中各个属性的根源属性应该合理分配，避免某一属性影响力过大
///
/// ## 受伤上限
///
/// 受伤上限 本质即 生命值和护盾值
///
/// - 生命值 [`Health`]
///   - 直接正相关 [`Strength`] [`damage_system::calc_health_max`]
/// - 替身护盾 [`ShieldSubstitute`]
///   - 直接正相关 [`Belief`] todo 信念超过阈值才能激发替身护盾
/// - 防护护盾 [`ShieldDefence`]
///   - 直接正相关 [`ArmorHard`] [`damage_system::calc_defence_shield`]
///   - 间接正相关 [`Strength`] todo 数值上盔甲坚韧与质量呈正相关，气力决定可穿戴质量，因此可以近似取代
/// - 奥术护盾 [`ShieldArcane`]
///   - 直接正相关 [`Belief`] todo
///
/// 不同 伤害类型 [`DamageType`] 对应的 受伤上限 见 [`damage_system::apply_damages`]
///
/// - 真实伤害 [`DamageType::KarmaTruth`]
///   - 伤害 [`Health`]
/// - 物理剪切 [`DamageType::PhysicsShear`]
///   - 伤害 [`Health`] [`ShieldSubstitute`] [`ShieldDefence`]
/// - 物理冲击 [`DamageType::PhysicsImpact`]
///   - 伤害 [`Health`] [`ShieldSubstitute`]
/// - 魔法奥术 [`DamageType::MagickaArcane`]
///   - 伤害 [`Health`] [`ShieldSubstitute`] [`ShieldArcane`]
/// - 不考虑破盾伤害，与上面相似
///
/// ## 伤害成长
///
/// 不同 伤害类型 [`DamageType`] 对应的 伤害缩放 见 [`damage_system::calc_damage_scale`]
///
/// - 真实伤害 [`DamageType::KarmaTruth`]
///   - 为招式固有属性，与角色收获相关，使用内禀属性代替
///   - 间接正相关 [`Strength`] or [`Belief`]
/// - 物理剪切 [`DamageType::PhysicsShear`]
///   - 直接正相关 [`Strength`] * [`WeaponSharp`]
///   - 其中 [`WeaponSharp`] 为武器固有属性，随角色成长增长，但是设计边际递减
///   - 近似正相关 [`Strength`]
/// - 物理冲击 [`DamageType::PhysicsImpact`]
///   - 直接正相关 [`Strength`] * [`WeaponMass`] / [`ArmorSoft`]
///   - 其中 [`WeaponMass`] 和 [`ArmorSoft`] 均为武器盔甲固有属性，都设计边际递减
///   - 近似正相关 [`Strength`]
/// - 魔法奥术 [`DamageType::MagickaArcane`]
///   - 直接正相关 [`Belief`]
///
/// 伤害成长 与 受伤上限 数值平衡分析（玩家受击角度）
/// （根据伤害类型找到针对的资源条、再找到相关的成长属性，对比伤害成长来源，二者是否能相互抵消）
///
/// - 真实伤害 [`DamageType::KarmaTruth`]
///   - 受伤上限 正相关 [`Strength`]
///   - 伤害成长 正相关 [`Strength`] or [`Belief`]
///   - 对于 [`Strength`] 成长是平衡的
///   - 对于 [`Belief`] 成长【受击者不利】，算作差异性，不在此系统弥补
///   - 由于其不平衡性，应注意避免数值膨胀，并在其他机制弥补，如：替死法术、冲击韧性机制、远程拉扯等
/// - 物理剪切 [`DamageType::PhysicsShear`]
///   - 受伤上限 正相关 [`Strength`] * 2 + [`Belief`]
///   - 伤害成长 正相关 [`Strength`]
///   - 对于 [`Strength`] 成长是平衡的
///   - 对于 [`Belief`] 成长【攻击者不利】，可令法术附带该类伤害
/// - 物理冲击 [`DamageType::PhysicsImpact`]
///   - 受伤上限 正相关 [`Strength`] + [`Belief`]
///   - 伤害成长 正相关 [`Strength`]
///   - 对于 [`Strength`] 成长是平衡的
///   - 对于 [`Belief`] 成长【攻击者不利】，可令法术附带该类伤害
/// - 魔法奥术 [`DamageType::MagickaArcane`]
///   - 受伤上限 正相关 [`Belief`] * 2 + [`Strength`]
///   - 伤害成长 正相关 [`Belief`]
///   - 对于 [`Strength`] 成长【攻击者不利】，可令武器附带该类伤害
///   - 对于 [`Belief`] 成长是平衡的
pub mod damage_system {
    use super::*;

    const MAGICKA_BASELINE: f64 = 100.0;

    /// 每帧计算伤害前都先进行同类合并
    pub fn merge_damages<S: FixedName>(
        damage_buffer: &mut DamageEffectBuffer<S>,
        target_health: &Health,
        target_shield_defence: &ShieldDefence,
        target_shield_arcane: &ShieldArcane,
    ) -> [(DamageType, Option<Effect<S>>); 6] {
        let mut dmg_km_truth: (_, Option<Effect<S>>) = (DamageType::KarmaTruth, None);
        let mut dmg_phy_shear: (_, Option<Effect<S>>) = (DamageType::PhysicsShear, None);
        let mut dmg_phy_impact: (_, Option<Effect<S>>) = (DamageType::PhysicsImpact, None);
        let mut dmg_mgk_arcane: (_, Option<Effect<S>>) = (DamageType::MagickaArcane, None);
        let mut dmg_bk_sld_defence: (_, Option<Effect<S>>) = (DamageType::BrokeShieldDefence, None);
        let mut dmg_bk_sld_arcane: (_, Option<Effect<S>>) = (DamageType::BrokeShieldArcane, None);

        // get the ownership
        for dmg_eff in damage_buffer.0.drain(0..) {
            let DamageEffect {
                dmg_type,
                dmg_calc,
                mut eff,
            } = dmg_eff;

            // 根据伤害类型找到聚合对象
            let merged_dmg = match dmg_type {
                DamageType::KarmaTruth => &mut dmg_km_truth.1,
                DamageType::PhysicsShear => &mut dmg_phy_shear.1,
                DamageType::PhysicsImpact => &mut dmg_phy_impact.1,
                DamageType::MagickaArcane => &mut dmg_mgk_arcane.1,
                DamageType::BrokeShieldDefence => &mut dmg_bk_sld_defence.1,
                DamageType::BrokeShieldArcane => &mut dmg_bk_sld_arcane.1,
            };

            // 提前获取原始效果值
            let origin_eff_val = eff.get_effect_value();
            // 预处理聚合对象，移走所有权
            if merged_dmg.is_none() {
                eff.set_effect_value(0.0);
                *merged_dmg = Some(eff);
            }

            // 根据伤害类型找到百分比参照物
            let base_prop = match dmg_type {
                DamageType::KarmaTruth
                | DamageType::PhysicsShear
                | DamageType::PhysicsImpact
                | DamageType::MagickaArcane => &target_health.0,
                DamageType::BrokeShieldDefence => &target_shield_defence.0,
                DamageType::BrokeShieldArcane => &target_shield_arcane.0,
            };

            // 根据伤害算法计算伤害绝对值
            let abs_eff_val = match dmg_calc {
                DamageCalc::Val => origin_eff_val,
                DamageCalc::CurPer => origin_eff_val * base_prop.get_current(),
                DamageCalc::MaxPer => origin_eff_val * base_prop.get_max(),
            };

            // 累加绝对值
            if let Some(merged_dmg) = merged_dmg {
                merged_dmg.set_effect_value(merged_dmg.get_effect_value() + abs_eff_val);
            }
        }

        // 破盾伤害优先计算，然后是有防护伤害，最后是真实伤害
        [
            dmg_bk_sld_defence,
            dmg_bk_sld_arcane,
            dmg_mgk_arcane,
            dmg_phy_shear,
            dmg_phy_impact,
            dmg_km_truth,
        ]
    }

    pub struct DamageAppliedAttrProps<'a> {
        pub source_strength: &'a Strength,
        pub source_belief: &'a Belief,
        pub source_magicka: &'a Magicka,
        pub source_weapon_sharp: &'a WeaponSharp,
        pub source_weapon_mass: &'a WeaponMass,
        pub target_armor_soft: &'a ArmorSoft,
    }

    /// 对合并后的伤害效果计算伤害
    pub fn apply_damages<S: FixedName>(
        dmg_effs: [(DamageType, Option<Effect<S>>); 6],
        damage_applied_attr_props: DamageAppliedAttrProps,
        target_health: &mut Health,
        target_shield_substitute: &mut ShieldSubstitute,
        target_shield_defence: &mut ShieldDefence,
        target_shield_arcane: &mut ShieldArcane,
    ) -> DamageInfo<S> {
        let DamageAppliedAttrProps {
            source_strength,
            source_belief,
            source_magicka,
            source_weapon_sharp,
            source_weapon_mass,
            target_armor_soft,
        } = damage_applied_attr_props;

        let mut dmg_info: DamageInfo<S> = DamageInfo::default();
        for (dmg_type, dmg_eff) in dmg_effs {
            if let Some(dmg_eff) = dmg_eff {
                let target_props: &mut [&mut Prop] = match dmg_type {
                    DamageType::KarmaTruth => &mut [&mut target_health.0],
                    DamageType::PhysicsShear => &mut [
                        &mut target_shield_defence.0,
                        &mut target_shield_substitute.0,
                        &mut target_health.0,
                    ],
                    DamageType::PhysicsImpact => {
                        &mut [&mut target_shield_substitute.0, &mut target_health.0]
                    }
                    DamageType::MagickaArcane => &mut [
                        &mut target_shield_arcane.0,
                        &mut target_shield_substitute.0,
                        &mut target_health.0,
                    ],
                    DamageType::BrokeShieldDefence => &mut [&mut target_shield_defence.0],
                    DamageType::BrokeShieldArcane => &mut [&mut target_shield_arcane.0],
                };

                // 根据伤害类型计算缩放比例
                let dmg_scale = damage_system::calc_damage_scale(
                    dmg_type,
                    source_strength,
                    source_belief,
                    source_magicka,
                    source_weapon_sharp,
                    source_weapon_mass,
                    target_armor_soft,
                );

                let mut real_dmg = dmg_scale * dmg_eff.get_effect_value();
                for prop in target_props {
                    let res = prop.apply_eff(real_dmg);
                    real_dmg -= res.real_eff_val;
                }

                if dmg_info.first_hurt_heal_from_eff.is_none() && dmg_type.is_hurt_heal() {
                    dmg_info.first_hurt_heal_from_eff = Some(dmg_eff.own_from_eff_name());
                }
            }
        }

        dmg_info
    }

    /// 伤害缩放
    pub fn calc_damage_scale(
        dmg_type: DamageType,
        source_strength: &Strength,
        source_belief: &Belief,
        source_magicka: &Magicka,
        source_weapon_sharp: &WeaponSharp,
        source_weapon_mass: &WeaponMass,
        target_armor_soft: &ArmorSoft,
    ) -> f64 {
        let damage_scale = match dmg_type {
            DamageType::KarmaTruth => 1.0,
            DamageType::PhysicsShear | DamageType::BrokeShieldDefence => {
                source_strength.0.get_current() * source_weapon_sharp.0.get_current()
            }
            DamageType::PhysicsImpact => {
                (source_strength.0.get_current() + source_weapon_mass.0.get_current())
                    / target_armor_soft.0.get_current()
            }
            DamageType::MagickaArcane | DamageType::BrokeShieldArcane => {
                source_belief.0.get_current()
            }
        };

        // 能量越高伤害越高 不使用双方能量差是为了防止在高能量状态下，小怪低能量形成的碾压，导致堆怪没威胁
        let base_scale = 0.0_f64.max(1.0 + source_magicka.0.get_current() / MAGICKA_BASELINE);

        damage_scale * base_scale
    }

    /// [`Strength`] 影响 [`Health`]
    pub fn calc_health_max(health_base: f64, health_scale: f64, strength: &Strength) -> f64 {
        health_base + health_scale * strength.0.get_origin()
    }

    /// [`Belief`] 影响【原始能量】
    pub fn calc_magicka_value(magicka_base: f64, magicka_scale: f64, belief: &Belief) -> f64 {
        magicka_base + magicka_scale * belief.0.get_origin()
    }

    /// 【原始能量】影响 [`Magicka`]
    pub fn calc_magicka_max(
        magicka_base: f64,
        magicka_scale: f64,
        belief: &Belief,
        magicka_energy_level: &MagickaEnergyLevel,
    ) -> f64 {
        let magicka_value = calc_magicka_value(magicka_base, magicka_scale, belief);
        magicka_energy_level.max_energy(magicka_value)
    }

    /// [`ArmorHard`] 影响 [`ShieldDefence`]
    pub fn calc_defence_shield(armor_hard: &ArmorHard) -> f64 {
        armor_hard.0.get_current()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base_lib::eff_attr_prop::attrs::Attr;

    /// 等级 A 演示：通过公开构造路径喂入伤害并合并
    ///
    /// 完全使用公开 API（`DamageEffect::new` + `DamageEffectBuffer::push`），
    /// 验证上层无需访问私有字段即可驱动 `merge_damages`。
    #[test]
    fn test_damage_pipeline_constructible_merge() {
        let mut buffer = DamageEffectBuffer::new();
        buffer.push(DamageEffect::new(
            DamageType::PhysicsShear,
            DamageCalc::Val,
            Effect::new("source", "dmg", -10.0),
        ));
        buffer.push(DamageEffect::new(
            DamageType::PhysicsShear,
            DamageCalc::Val,
            Effect::new("source", "dmg", -5.0),
        ));
        assert_eq!(buffer.len(), 2);

        let target_health = Health(Prop::new(100.0, 100.0, 0.0));
        let target_shield_defence = ShieldDefence(Prop::new(50.0, 50.0, 0.0));
        let target_shield_arcane = ShieldArcane(Prop::new(50.0, 50.0, 0.0));

        let merged = damage_system::merge_damages(
            &mut buffer,
            &target_health,
            &target_shield_defence,
            &target_shield_arcane,
        );

        // 合并数组顺序：[破防护盾, 破奥术盾, 奥术, 剪切, 冲击, 真实]，剪切在下标 3
        let merged_shear = merged[3].1.as_ref().expect("物理剪切伤害应被合并");
        assert_eq!(merged_shear.get_effect_value(), -15.0);
    }

    /// 等级 A 演示：merge → apply 全链路可由公开构造路径驱动
    #[test]
    fn test_damage_pipeline_constructible_apply() {
        let mut buffer = DamageEffectBuffer::new();
        buffer.push(DamageEffect::new(
            DamageType::KarmaTruth,
            DamageCalc::Val,
            Effect::new("source", "dmg", -10.0),
        ));

        let mut target_health = Health(Prop::new(100.0, 100.0, 0.0));
        let mut target_shield_substitute = ShieldSubstitute(Prop::new(50.0, 50.0, 0.0));
        let mut target_shield_defence = ShieldDefence(Prop::new(50.0, 50.0, 0.0));
        let mut target_shield_arcane = ShieldArcane(Prop::new(50.0, 50.0, 0.0));

        let merged = damage_system::merge_damages(
            &mut buffer,
            &target_health,
            &target_shield_defence,
            &target_shield_arcane,
        );

        let attr_props = damage_system::DamageAppliedAttrProps {
            source_strength: &Strength(Attr::new(1.0)),
            source_belief: &Belief(Attr::new(1.0)),
            source_magicka: &Magicka(Prop::new(0.0, 0.0, 0.0)),
            source_weapon_sharp: &WeaponSharp(Attr::new(1.0)),
            source_weapon_mass: &WeaponMass(Attr::new(1.0)),
            target_armor_soft: &ArmorSoft(Attr::new(1.0)),
        };

        let dmg_info = damage_system::apply_damages(
            merged,
            attr_props,
            &mut target_health,
            &mut target_shield_substitute,
            &mut target_shield_defence,
            &mut target_shield_arcane,
        );

        // 真实伤害只作用于血量：100 - 10
        assert_eq!(target_health.0.get_current(), 90.0);
        // 死因记录到首个造成伤害的效果
        let (from, eff) = dmg_info.first_hurt_heal_from_eff.expect("应有死因记录");
        assert_eq!(from, "source");
        assert_eq!(eff, "dmg");
    }
}

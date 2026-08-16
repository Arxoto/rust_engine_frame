use crate::{
    base_lib::{
        cores::unify_types::FixedName,
        eff_attr_prop::{effects::Effect, prop_alter_eff::PropAlterEffectType, props::Prop},
    },
    common_impl::combats::{
        combat_additions::{ArmorHard, ArmorSoft, WeaponMass, WeaponSharp},
        combat_inherents::{Belief, Strength},
        combat_units::{Health, Magicka, ShieldArcane, ShieldDefence, ShieldSubstitute},
    },
};

/// 存放伤害或治疗效果的 buffer
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
    /// 伤害类型，伤害针对的哪些目标
    dmg_type: DamageType,
    /// 伤害生效方式（绝对值或是百分比）
    ///
    /// 与 [`crate::base_lib::eff_attr_prop::prop_alter_eff::PropAlterEffect`] 共享同一计算语义
    eff_type: PropAlterEffectType,
    eff: Effect<S>,
}

impl<S: FixedName> DamageEffect<S> {
    /// 构造单次伤害效果
    ///
    /// 推入 [`DamageEffectBuffer`] 后由伤害系统消费。
    #[must_use]
    pub fn new(dmg_type: DamageType, eff_type: PropAlterEffectType, eff: Effect<S>) -> Self {
        Self {
            dmg_type,
            eff_type,
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
    /// 仅作用于生命值，可用作真实伤害或治疗
    OnlyHealth,
    /// 仅作用于替身护盾 （适用于添加护盾或破盾伤害，仅对本层护盾生效）
    OnlyShieldSubstitute,
    /// 仅作用于防护护盾 （适用于添加护盾或破盾伤害，仅对本层护盾生效）
    OnlyShieldDefence,
    /// 仅作用于奥术护盾 （适用于添加护盾或破盾伤害，仅对本层护盾生效）
    OnlyShieldArcane,

    /// 物理冲击 沉重
    PhysicsImpact,
    /// 物理剪切 尖锐
    PhysicsShears,
    /// 魔法奥术
    MagickaArcane,
}

impl<S: FixedName> Default for DamageInfo<S> {
    fn default() -> Self {
        Self {
            first_hurt_heal_from_eff: None,
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
/// - 真实伤害 [`DamageType::OnlyHealth`]
///   - 伤害 [`Health`]
/// - 物理剪切 [`DamageType::PhysicsShears`]
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
/// - 真实伤害 [`DamageType::OnlyHealth`]
///   - 为招式固有属性，与角色收获相关，使用内禀属性代替
///   - 间接正相关 [`Strength`] or [`Belief`]
/// - 物理剪切 [`DamageType::PhysicsShears`]
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
/// - 真实伤害 [`DamageType::OnlyHealth`]
///   - 受伤上限 正相关 [`Strength`]
///   - 伤害成长 正相关 [`Strength`] or [`Belief`]
///   - 对于 [`Strength`] 成长是平衡的
///   - 对于 [`Belief`] 成长【受击者不利】，算作差异性，不在此系统弥补
///   - 由于其不平衡性，应注意避免数值膨胀，并在其他机制弥补，如：替死法术、冲击韧性机制、远程拉扯等
/// - 物理剪切 [`DamageType::PhysicsShears`]
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
    use crate::common_impl::combats::combat_units::PropAboutDamageType;

    use super::*;

    const MAGICKA_BASELINE: f64 = 100.0;

    impl DamageType {
        /// 不同伤害类型 对哪些资源进行伤害
        ///
        /// 为明确业务逻辑，做以下约束
        /// - 若返回不是单元素，那么里面元素的 [`PropAboutDamageType::order_val`] 必须依次连续下降
        /// - 如 [2, 1, 0] / [1, 0] ，而不是 [2, 0]（不连续） [2, 2]（没有下降）
        pub fn target_types(&self) -> &[PropAboutDamageType] {
            match self {
                DamageType::OnlyHealth => &[PropAboutDamageType::Health],
                DamageType::OnlyShieldSubstitute => &[PropAboutDamageType::ShieldSubstitute],
                DamageType::OnlyShieldDefence => &[PropAboutDamageType::ShieldDefence],
                DamageType::OnlyShieldArcane => &[PropAboutDamageType::ShieldArcane],
                DamageType::PhysicsImpact => &[
                    PropAboutDamageType::ShieldSubstitute,
                    PropAboutDamageType::Health,
                ],
                DamageType::PhysicsShears => &[
                    PropAboutDamageType::ShieldDefence,
                    PropAboutDamageType::ShieldSubstitute,
                    PropAboutDamageType::Health,
                ],
                DamageType::MagickaArcane => &[
                    PropAboutDamageType::ShieldArcane,
                    PropAboutDamageType::ShieldSubstitute,
                    PropAboutDamageType::Health,
                ],
            }
        }

        /// 返回伤害计算的排序依据（索引位置），以保证同一帧内伤害计算的顺序无关性
        ///
        ///
        /// 返回值满足
        /// - 返回值必定不重复，且在 `[0, n)` 之间，其中 n 为 [`DamageType`] 类型的个数
        /// - 返回值是基于 [`Self::target_types`] 的返回值确定的，具体规则如下
        ///   - 单元素必定排在多元素的前面
        ///   - 对比首个元素的 [`PropAboutDamageType::order_val`] 值大的在前面（无需依次对比后面的元素，因为 [`Self::target_types`] 的返回值约束）
        pub fn order_val(&self) -> usize {
            match self {
                DamageType::OnlyHealth => 0,
                DamageType::OnlyShieldSubstitute => 1,
                DamageType::OnlyShieldDefence => 2,
                DamageType::OnlyShieldArcane => 3,
                DamageType::PhysicsImpact => 4,
                DamageType::PhysicsShears => 5,
                DamageType::MagickaArcane => 6,
            }
        }

        /// 百分比伤害计算时选取哪个资源为基础进行计算
        ///
        /// 返回 [`Self::target_types`] 的最后一个元素
        pub fn percent_base_type(&self) -> PropAboutDamageType {
            match self {
                DamageType::OnlyHealth
                | DamageType::PhysicsImpact
                | DamageType::PhysicsShears
                | DamageType::MagickaArcane => PropAboutDamageType::Health,
                DamageType::OnlyShieldSubstitute => PropAboutDamageType::ShieldSubstitute,
                DamageType::OnlyShieldDefence => PropAboutDamageType::ShieldDefence,
                DamageType::OnlyShieldArcane => PropAboutDamageType::ShieldArcane,
            }
        }

        /// 能否对血量造成伤害
        ///
        /// 依据是 [`Self::target_types`] 的最后一个元素是否 [`PropAboutDamageType::Health`]
        pub fn is_hurt_heal(&self) -> bool {
            match self {
                DamageType::OnlyHealth => true,
                DamageType::PhysicsImpact => true,
                DamageType::PhysicsShears => true,
                DamageType::MagickaArcane => true,
                DamageType::OnlyShieldSubstitute => false,
                DamageType::OnlyShieldDefence => false,
                DamageType::OnlyShieldArcane => false,
            }
        }
    }

    #[derive(Debug)]
    pub struct MergedDamageEffs<S: FixedName> {
        dmg_only_heal: Option<Effect<S>>,
        dmg_only_sub: Option<Effect<S>>,
        dmg_only_def: Option<Effect<S>>,
        dmg_only_arc: Option<Effect<S>>,
        dmg_phy_imp: Option<Effect<S>>,
        dmg_phy_she: Option<Effect<S>>,
        dmg_mgk_arc: Option<Effect<S>>,
    }

    impl<S: FixedName> Default for MergedDamageEffs<S> {
        fn default() -> Self {
            Self {
                dmg_only_heal: None,
                dmg_only_sub: None,
                dmg_only_def: None,
                dmg_only_arc: None,
                dmg_phy_imp: None,
                dmg_phy_she: None,
                dmg_mgk_arc: None,
            }
        }
    }

    type MergedDamageEffArray<S> = [(DamageType, Option<Effect<S>>); 7];

    impl<S: FixedName> MergedDamageEffs<S> {
        /// 顺序与 [`DamageType::order_val`] 一样
        pub fn into_slice(self) -> MergedDamageEffArray<S> {
            [
                (DamageType::OnlyHealth, self.dmg_only_heal),
                (DamageType::OnlyShieldSubstitute, self.dmg_only_sub),
                (DamageType::OnlyShieldDefence, self.dmg_only_def),
                (DamageType::OnlyShieldArcane, self.dmg_only_arc),
                (DamageType::PhysicsImpact, self.dmg_phy_imp),
                (DamageType::PhysicsShears, self.dmg_phy_she),
                (DamageType::MagickaArcane, self.dmg_mgk_arc),
            ]
        }
    }

    /// 每帧计算伤害前都先进行同类合并
    ///
    /// 合并方便伤害计算，具体原因如下
    /// - 若先【物理伤害】，后【破盾伤害】，那么当两者加起来能够破盾时，实际伤害与顺序有关
    /// - 【物理伤害】在前会导致后面的【破盾伤害】无效化
    ///
    /// 因此得出结论：针对单一资源的伤害必须在针对复合资源的伤害之前结算，具体见 [`DamageType::order_val`]
    pub fn merge_damages<S: FixedName>(
        damage_buffer: &mut DamageEffectBuffer<S>,
        target_health: &Health,
        target_shield_substitute: &ShieldSubstitute,
        target_shield_defence: &ShieldDefence,
        target_shield_arcane: &ShieldArcane,
    ) -> MergedDamageEffs<S> {
        let mut merged_dmg_effs = MergedDamageEffs::<S>::default();

        // get the ownership
        for dmg_eff in damage_buffer.0.drain(0..) {
            let DamageEffect {
                dmg_type,
                eff_type,
                mut eff,
            } = dmg_eff;

            // 根据伤害类型找到聚合对象
            let merged_dmg = match dmg_type {
                DamageType::OnlyHealth => &mut merged_dmg_effs.dmg_only_heal,
                DamageType::OnlyShieldSubstitute => &mut merged_dmg_effs.dmg_only_sub,
                DamageType::OnlyShieldDefence => &mut merged_dmg_effs.dmg_only_def,
                DamageType::OnlyShieldArcane => &mut merged_dmg_effs.dmg_only_arc,
                DamageType::PhysicsImpact => &mut merged_dmg_effs.dmg_phy_imp,
                DamageType::PhysicsShears => &mut merged_dmg_effs.dmg_phy_she,
                DamageType::MagickaArcane => &mut merged_dmg_effs.dmg_mgk_arc,
            };

            // 提前获取原始效果值
            let origin_eff_val = eff.get_effect_value();
            // 预处理聚合对象，移走所有权
            if merged_dmg.is_none() {
                eff.set_effect_value(0.0);
                *merged_dmg = Some(eff);
            }

            // 根据伤害类型找到百分比参照物
            let base_prop = match dmg_type.percent_base_type() {
                PropAboutDamageType::Health => &target_health.0,
                PropAboutDamageType::ShieldSubstitute => &target_shield_substitute.0,
                PropAboutDamageType::ShieldDefence => &target_shield_defence.0,
                PropAboutDamageType::ShieldArcane => &target_shield_arcane.0,
            };

            // 根据伤害算法计算伤害绝对值
            let abs_eff_val = match eff_type {
                PropAlterEffectType::Val => origin_eff_val,
                PropAlterEffectType::CurPer => origin_eff_val * base_prop.get_current(),
                PropAlterEffectType::MaxPer => origin_eff_val * base_prop.get_max(),
            };

            // 累加绝对值
            if let Some(merged_dmg) = merged_dmg {
                merged_dmg.set_effect_value(merged_dmg.get_effect_value() + abs_eff_val);
            }
        }

        merged_dmg_effs
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
        merged_dmg_effs: MergedDamageEffs<S>,
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
        let dmg_effs = merged_dmg_effs.into_slice();
        for (dmg_type, dmg_eff) in dmg_effs {
            if let Some(dmg_eff) = dmg_eff {
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
                for target_prop_type in dmg_type.target_types() {
                    let prop = match target_prop_type {
                        PropAboutDamageType::Health => &mut target_health.0,
                        PropAboutDamageType::ShieldSubstitute => &mut target_shield_substitute.0,
                        PropAboutDamageType::ShieldDefence => &mut target_shield_defence.0,
                        PropAboutDamageType::ShieldArcane => &mut target_shield_arcane.0,
                    };
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
        // 真实伤害与护盾专精不受能量加成
        let damage_scale = match dmg_type {
            DamageType::OnlyHealth
            | DamageType::OnlyShieldSubstitute
            | DamageType::OnlyShieldDefence
            | DamageType::OnlyShieldArcane => {
                return 1.0;
            }
            DamageType::PhysicsImpact => {
                (source_strength.0.get_current() + source_weapon_mass.0.get_current())
                    / target_armor_soft.0.get_current()
            }
            DamageType::PhysicsShears => {
                source_strength.0.get_current() * source_weapon_sharp.0.get_current()
            }
            DamageType::MagickaArcane => source_belief.0.get_current(),
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
            DamageType::PhysicsShears,
            PropAlterEffectType::Val,
            Effect::new("source", "dmg", -10.0),
        ));
        buffer.push(DamageEffect::new(
            DamageType::PhysicsShears,
            PropAlterEffectType::Val,
            Effect::new("source", "dmg", -5.0),
        ));
        assert_eq!(buffer.len(), 2);

        let target_health = Health(Prop::new(100.0, 100.0, 0.0));
        let target_shield_defence = ShieldDefence(Prop::new(50.0, 50.0, 0.0));
        let target_shield_arcane = ShieldArcane(Prop::new(50.0, 50.0, 0.0));
        let target_shield_substitute = ShieldSubstitute(Prop::new(50.0, 50.0, 0.0));

        let merged = damage_system::merge_damages(
            &mut buffer,
            &target_health,
            &target_shield_defence,
            &target_shield_arcane,
            &target_shield_substitute,
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
            DamageType::OnlyHealth,
            PropAlterEffectType::Val,
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
            &target_shield_substitute,
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

    // region: 等级 B 演示测试

    /// 在合并结果中按伤害类型查找效果(局部 helper,避免魔法下标)
    fn find_eff<S: FixedName>(
        merged: &[(DamageType, Option<Effect<S>>)],
        target: DamageType,
    ) -> Option<&Effect<S>> {
        merged.iter().find_map(|(t, eff)| {
            (std::mem::discriminant(t) == std::mem::discriminant(&target))
                .then_some(eff.as_ref())
                .flatten()
        })
    }

    /// 合并数组顺序是解析优先级契约:破盾优先 → 有防护 → 真实(见 docs/adr/0001)
    #[test]
    fn merge_damages_array_order_is_resolution_priority() {
        let mut buffer = DamageEffectBuffer::new();
        // 入队顺序刻意与解析顺序相反,验证按类型归位而非按入队序
        buffer.push(DamageEffect::new(
            DamageType::OnlyHealth,
            PropAlterEffectType::Val,
            Effect::new("s", "e", -1.0),
        ));
        buffer.push(DamageEffect::new(
            DamageType::MagickaArcane,
            PropAlterEffectType::Val,
            Effect::new("s", "e", -1.0),
        ));
        buffer.push(DamageEffect::new(
            DamageType::PhysicsImpact,
            PropAlterEffectType::Val,
            Effect::new("s", "e", -1.0),
        ));
        buffer.push(DamageEffect::new(
            DamageType::PhysicsShears,
            PropAlterEffectType::Val,
            Effect::new("s", "e", -1.0),
        ));
        buffer.push(DamageEffect::new(
            DamageType::BrokeShieldArcane,
            PropAlterEffectType::Val,
            Effect::new("s", "e", -1.0),
        ));
        buffer.push(DamageEffect::new(
            DamageType::BrokeShieldDefence,
            PropAlterEffectType::Val,
            Effect::new("s", "e", -1.0),
        ));

        let target_health = Health(Prop::new(100.0, 100.0, 0.0));
        let target_shield_defence = ShieldDefence(Prop::new(50.0, 50.0, 0.0));
        let target_shield_arcane = ShieldArcane(Prop::new(50.0, 50.0, 0.0));
        let target_shield_substitute = ShieldSubstitute(Prop::new(50.0, 50.0, 0.0));

        let merged = damage_system::merge_damages(
            &mut buffer,
            &target_health,
            &target_shield_defence,
            &target_shield_arcane,
            &target_shield_substitute,
        );

        // 契约顺序:前 6 项为 破防护盾 → 破奥术盾 → 奥术 → 剪切 → 冲击 → 真实,
        // 后 3 项为仅护盾装载(见 docs/adr/0001)
        let order: Vec<_> = merged
            .iter()
            .take(6)
            .map(|(t, _)| std::mem::discriminant(t))
            .collect();
        let expected: Vec<_> = [
            DamageType::BrokeShieldDefence,
            DamageType::BrokeShieldArcane,
            DamageType::MagickaArcane,
            DamageType::PhysicsShears,
            DamageType::PhysicsImpact,
            DamageType::OnlyHealth,
        ]
        .iter()
        .map(std::mem::discriminant)
        .collect();
        assert_eq!(order, expected);
        // 后 3 项:仅护盾装载追加末尾
        let tail: Vec<_> = merged
            .iter()
            .skip(6)
            .map(|(t, _)| std::mem::discriminant(t))
            .collect();
        let expected_tail: Vec<_> = [
            DamageType::OnlyShieldDefence,
            DamageType::OnlyShieldSubstitute,
            DamageType::OnlyShieldArcane,
        ]
        .iter()
        .map(std::mem::discriminant)
        .collect();
        assert_eq!(tail, expected_tail);
    }

    /// merge 按类型聚合,并可按 DamageType 查找
    #[test]
    fn merge_aggregates_by_type_and_lookup() {
        let mut buffer = DamageEffectBuffer::new();
        buffer.push(DamageEffect::new(
            DamageType::PhysicsShears,
            PropAlterEffectType::Val,
            Effect::new("s", "e", -5.0),
        ));
        buffer.push(DamageEffect::new(
            DamageType::OnlyHealth,
            PropAlterEffectType::Val,
            Effect::new("s", "e", -3.0),
        ));
        buffer.push(DamageEffect::new(
            DamageType::PhysicsShears,
            PropAlterEffectType::Val,
            Effect::new("s", "e", -7.0),
        ));

        let target_health = Health(Prop::new(100.0, 100.0, 0.0));
        let target_shield_defence = ShieldDefence(Prop::new(50.0, 50.0, 0.0));
        let target_shield_arcane = ShieldArcane(Prop::new(50.0, 50.0, 0.0));
        let target_shield_substitute = ShieldSubstitute(Prop::new(50.0, 50.0, 0.0));

        let merged = damage_system::merge_damages(
            &mut buffer,
            &target_health,
            &target_shield_defence,
            &target_shield_arcane,
            &target_shield_substitute,
        );

        let shear = find_eff(&merged, DamageType::PhysicsShears).expect("剪切应被合并");
        assert_eq!(shear.get_effect_value(), -12.0);
        let truth = find_eff(&merged, DamageType::OnlyHealth).expect("真实应被合并");
        assert_eq!(truth.get_effect_value(), -3.0);
    }

    /// 喂入单次伤害(标准攻击者:缩放相关属性均为 1、能量 0),返回 (DamageInfo, health, sub, def, arc)
    fn apply_one(
        dmg_type: DamageType,
        eff_type: PropAlterEffectType,
        eff_val: f64,
        health: Prop,
        sub: Prop,
        def: Prop,
        arc: Prop,
    ) -> (DamageInfo<&'static str>, Prop, Prop, Prop, Prop) {
        let mut buffer = DamageEffectBuffer::new();
        buffer.push(DamageEffect::new(
            dmg_type,
            eff_type,
            Effect::new("source", "dmg", eff_val),
        ));

        let mut health = Health(health);
        let mut sub = ShieldSubstitute(sub);
        let mut def = ShieldDefence(def);
        let mut arc = ShieldArcane(arc);

        let merged = damage_system::merge_damages(&mut buffer, &health, &def, &arc, &sub);
        let attr_props = damage_system::DamageAppliedAttrProps {
            source_strength: &Strength(Attr::new(1.0)),
            source_belief: &Belief(Attr::new(1.0)),
            source_magicka: &Magicka(Prop::new(0.0, 0.0, 0.0)),
            source_weapon_sharp: &WeaponSharp(Attr::new(1.0)),
            source_weapon_mass: &WeaponMass(Attr::new(1.0)),
            target_armor_soft: &ArmorSoft(Attr::new(1.0)),
        };
        let info = damage_system::apply_damages(
            merged,
            attr_props,
            &mut health,
            &mut sub,
            &mut def,
            &mut arc,
        );

        (info, health.0, sub.0, def.0, arc.0)
    }

    /// 真实伤害只打血
    #[test]
    fn karma_truth_hits_health_only() {
        let (_, health, sub, def, arc) = apply_one(
            DamageType::OnlyHealth,
            PropAlterEffectType::Val,
            -10.0,
            Prop::new(100.0, 100.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
        );
        assert_eq!(health.get_current(), 90.0);
        assert_eq!(sub.get_current(), 50.0);
        assert_eq!(def.get_current(), 50.0);
        assert_eq!(arc.get_current(), 50.0);
    }

    /// 剪切伤害打 防护盾 → 替身 → 血;小伤害被防护盾吸收
    #[test]
    fn physics_shear_hits_defence_substitute_health() {
        let (_, health, sub, def, arc) = apply_one(
            DamageType::PhysicsShears,
            PropAlterEffectType::Val,
            -10.0,
            Prop::new(100.0, 100.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
        );
        assert_eq!(def.get_current(), 40.0);
        assert_eq!(sub.get_current(), 50.0);
        assert_eq!(health.get_current(), 100.0);
        assert_eq!(arc.get_current(), 50.0);
    }

    /// 冲击伤害打 替身 → 血;缩放为 (气力 + 质量) / 柔韧 = 2
    #[test]
    fn physics_impact_hits_substitute_health() {
        let (_, health, sub, def, arc) = apply_one(
            DamageType::PhysicsImpact,
            PropAlterEffectType::Val,
            -10.0,
            Prop::new(100.0, 100.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
        );
        // 真实伤害 20,全部被替身吸收
        assert_eq!(sub.get_current(), 30.0);
        assert_eq!(health.get_current(), 100.0);
        assert_eq!(def.get_current(), 50.0);
        assert_eq!(arc.get_current(), 50.0);
    }

    /// 奥术伤害打 奥术盾 → 替身 → 血
    #[test]
    fn magicka_arcane_hits_arcane_substitute_health() {
        let (_, health, sub, def, arc) = apply_one(
            DamageType::MagickaArcane,
            PropAlterEffectType::Val,
            -10.0,
            Prop::new(100.0, 100.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
        );
        assert_eq!(arc.get_current(), 40.0);
        assert_eq!(sub.get_current(), 50.0);
        assert_eq!(health.get_current(), 100.0);
        assert_eq!(def.get_current(), 50.0);
    }

    /// 破防护盾只打防护盾
    #[test]
    fn broke_shield_defence_hits_defence_only() {
        let (_, health, sub, def, arc) = apply_one(
            DamageType::BrokeShieldDefence,
            PropAlterEffectType::Val,
            -10.0,
            Prop::new(100.0, 100.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
        );
        assert_eq!(def.get_current(), 40.0);
        assert_eq!(sub.get_current(), 50.0);
        assert_eq!(health.get_current(), 100.0);
        assert_eq!(arc.get_current(), 50.0);
    }

    /// 破奥术盾只打奥术盾
    #[test]
    fn broke_shield_arcane_hits_arcane_only() {
        let (_, health, sub, def, arc) = apply_one(
            DamageType::BrokeShieldArcane,
            PropAlterEffectType::Val,
            -10.0,
            Prop::new(100.0, 100.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
        );
        assert_eq!(arc.get_current(), 40.0);
        assert_eq!(sub.get_current(), 50.0);
        assert_eq!(health.get_current(), 100.0);
        assert_eq!(def.get_current(), 50.0);
    }

    /// 护盾命中次序:大剪切伤害按 防护盾 → 替身 → 血 递减
    #[test]
    fn shield_hit_order_drains_defence_then_substitute() {
        let (_, health, sub, def, arc) = apply_one(
            DamageType::PhysicsShears,
            PropAlterEffectType::Val,
            -80.0,
            Prop::new(100.0, 100.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
        );
        assert_eq!(def.get_current(), 0.0); // 防护盾先被清空
        assert_eq!(sub.get_current(), 20.0); // 余量 30 流向替身
        assert_eq!(health.get_current(), 100.0);
        assert_eq!(arc.get_current(), 50.0);
    }

    /// 死因记录「数组解析序第一个 hurt 类型」,而非入队顺序
    #[test]
    fn death_cause_follows_resolution_order_not_push_order() {
        let mut buffer = DamageEffectBuffer::new();
        buffer.push(DamageEffect::new(
            DamageType::OnlyHealth,
            PropAlterEffectType::Val,
            Effect::new("karma", "truth_dmg", -10.0),
        ));
        buffer.push(DamageEffect::new(
            DamageType::MagickaArcane,
            PropAlterEffectType::Val,
            Effect::new("magic", "arcane_dmg", -10.0),
        ));

        let mut health = Health(Prop::new(100.0, 100.0, 0.0));
        let mut sub = ShieldSubstitute(Prop::new(50.0, 50.0, 0.0));
        let mut def = ShieldDefence(Prop::new(50.0, 50.0, 0.0));
        let mut arc = ShieldArcane(Prop::new(50.0, 50.0, 0.0));

        let merged = damage_system::merge_damages(&mut buffer, &health, &def, &arc, &sub);
        let attr_props = damage_system::DamageAppliedAttrProps {
            source_strength: &Strength(Attr::new(1.0)),
            source_belief: &Belief(Attr::new(1.0)),
            source_magicka: &Magicka(Prop::new(0.0, 0.0, 0.0)),
            source_weapon_sharp: &WeaponSharp(Attr::new(1.0)),
            source_weapon_mass: &WeaponMass(Attr::new(1.0)),
            target_armor_soft: &ArmorSoft(Attr::new(1.0)),
        };
        let info = damage_system::apply_damages(
            merged,
            attr_props,
            &mut health,
            &mut sub,
            &mut def,
            &mut arc,
        );

        // 奥术在解析序先于真实,故死因记录奥术,而非先入队的真实
        let (from, eff) = info.first_hurt_heal_from_eff.expect("应有死因记录");
        assert_eq!(from, "magic");
        assert_eq!(eff, "arcane_dmg");
    }

    /// 死因记录不校验实际伤害量:护盾全吸收时仍记录首个 hurt 类型(观察,见 spec Comments)
    #[test]
    fn death_cause_records_even_when_shield_absorbs_all() {
        let (info, health, sub, def, _) = apply_one(
            DamageType::PhysicsShears,
            PropAlterEffectType::Val,
            -5.0,
            Prop::new(100.0, 100.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
            Prop::new(50.0, 50.0, 0.0),
        );
        assert_eq!(def.get_current(), 45.0); // 5 点全被防护盾吸收
        assert_eq!(sub.get_current(), 50.0);
        assert_eq!(health.get_current(), 100.0);
        assert!(info.first_hurt_heal_from_eff.is_some());
    }

    /// calc_damage_scale:各伤害类型缩放公式(期望值按 combats.rs 平衡文档手算)
    #[test]
    fn calc_damage_scale_formulas() {
        let strength = Strength(Attr::new(2.0));
        let belief = Belief(Attr::new(3.0));
        let magicka = Magicka(Prop::new(0.0, 0.0, 0.0));
        let sharp = WeaponSharp(Attr::new(4.0));
        let mass = WeaponMass(Attr::new(5.0));
        let armor_soft = ArmorSoft(Attr::new(2.0));

        let scale = |t| {
            damage_system::calc_damage_scale(
                t,
                &strength,
                &belief,
                &magicka,
                &sharp,
                &mass,
                &armor_soft,
            )
        };

        assert_eq!(scale(DamageType::OnlyHealth), 1.0); // 真实恒 1
        assert_eq!(scale(DamageType::PhysicsShears), 2.0 * 4.0); // 气力 * 锋利
        assert_eq!(scale(DamageType::PhysicsImpact), (2.0 + 5.0) / 2.0); // (气力 + 质量) / 柔韧
        assert_eq!(scale(DamageType::MagickaArcane), 3.0); // 信念
        assert_eq!(scale(DamageType::BrokeShieldDefence), 2.0 * 4.0); // 同剪切
        assert_eq!(scale(DamageType::BrokeShieldArcane), 3.0); // 同奥术
    }

    /// calc_damage_scale:能量越高伤害越高,基线 1 + magicka / 100
    #[test]
    fn calc_damage_scale_magicka_bonus() {
        let strength = Strength(Attr::new(1.0));
        let belief = Belief(Attr::new(1.0));
        let magicka = Magicka(Prop::new(50.0, 50.0, 0.0));
        let sharp = WeaponSharp(Attr::new(1.0));
        let mass = WeaponMass(Attr::new(1.0));
        let armor_soft = ArmorSoft(Attr::new(1.0));

        assert_eq!(
            damage_system::calc_damage_scale(
                DamageType::PhysicsShears,
                &strength,
                &belief,
                &magicka,
                &sharp,
                &mass,
                &armor_soft,
            ),
            1.5, // (1 * 1) * (1 + 50 / 100)
        );
    }

    /// calc_health_max:health_base + health_scale * 气力基础值
    #[test]
    fn calc_health_max_formula() {
        let strength = Strength(Attr::new(5.0));
        assert_eq!(
            damage_system::calc_health_max(100.0, 10.0, &strength),
            150.0
        );
    }

    /// calc_magicka_value / max:信念 → 原始能量 → 能级映射
    #[test]
    fn calc_magicka_formulas() {
        let belief = Belief(Attr::new(5.0));
        let level = MagickaEnergyLevel::new(100.0, 200.0, 300.0);

        // 原始能量 = 50 + 20 * 5 = 150
        assert_eq!(
            damage_system::calc_magicka_value(50.0, 20.0, &belief),
            150.0
        );
        // 介于 l0/l1 之间 → 抬到 l1
        assert_eq!(
            damage_system::calc_magicka_max(50.0, 20.0, &belief, &level),
            200.0
        );

        // 原始能量超过最高能级 → 取最高能级
        let high = Belief(Attr::new(20.0));
        assert_eq!(
            damage_system::calc_magicka_max(50.0, 20.0, &high, &level),
            300.0
        );
    }

    /// calc_defence_shield:盔甲坚韧即防护盾值
    #[test]
    fn calc_defence_shield_formula() {
        let armor_hard = ArmorHard(Attr::new(80.0));
        assert_eq!(damage_system::calc_defence_shield(&armor_hard), 80.0);
    }

    /// Only* 装载:仅作用于对应护盾,正值累加当前值,不伤血、不记死因
    #[test]
    fn only_shield_defence_loads_defence_only() {
        let mut buffer = DamageEffectBuffer::new();
        buffer.push(DamageEffect::new(
            DamageType::OnlyShieldDefence,
            PropAlterEffectType::Val,
            Effect::new("player", "shield_spell", 30.0),
        ));

        let mut health = Health(Prop::new(100.0, 100.0, 0.0));
        let mut sub = ShieldSubstitute(Prop::new(50.0, 50.0, 0.0));
        let mut def = ShieldDefence(Prop::new(20.0, 50.0, 0.0));
        let mut arc = ShieldArcane(Prop::new(50.0, 50.0, 0.0));

        let merged = damage_system::merge_damages(&mut buffer, &health, &def, &arc, &sub);
        let attr_props = damage_system::DamageAppliedAttrProps {
            source_strength: &Strength(Attr::new(1.0)),
            source_belief: &Belief(Attr::new(1.0)),
            source_magicka: &Magicka(Prop::new(50.0, 50.0, 0.0)),
            source_weapon_sharp: &WeaponSharp(Attr::new(1.0)),
            source_weapon_mass: &WeaponMass(Attr::new(1.0)),
            target_armor_soft: &ArmorSoft(Attr::new(1.0)),
        };
        let info = damage_system::apply_damages(
            merged,
            attr_props,
            &mut health,
            &mut sub,
            &mut def,
            &mut arc,
        );

        // 只作用于防护盾:20 + 30 = 50
        assert_eq!(def.0.get_current(), 50.0);
        // 其他资源不变
        assert_eq!(sub.0.get_current(), 50.0);
        assert_eq!(arc.0.get_current(), 50.0);
        assert_eq!(health.0.get_current(), 100.0);
        // 不记录死因
        assert!(info.first_hurt_heal_from_eff.is_none());
    }

    /// is_hurt_heal:Only* 与破盾类型同为 false(不伤血)
    #[test]
    fn only_shield_types_are_not_hurt_heal() {
        assert!(!DamageType::OnlyShieldDefence.is_hurt_heal());
        assert!(!DamageType::OnlyShieldSubstitute.is_hurt_heal());
        assert!(!DamageType::OnlyShieldArcane.is_hurt_heal());
    }

    /// calc_damage_scale:KarmaTruth 与 Only* 跳过能量加成(高能量仍返回纯缩放)
    #[test]
    fn calc_damage_scale_skips_energy_for_truth_and_only() {
        let strength = Strength(Attr::new(1.0));
        let belief = Belief(Attr::new(1.0));
        let magicka = Magicka(Prop::new(50.0, 50.0, 0.0)); // 高能量:若有加成应为 1.5
        let sharp = WeaponSharp(Attr::new(1.0));
        let mass = WeaponMass(Attr::new(1.0));
        let armor_soft = ArmorSoft(Attr::new(1.0));

        let scale = |t| {
            damage_system::calc_damage_scale(
                t,
                &strength,
                &belief,
                &magicka,
                &sharp,
                &mass,
                &armor_soft,
            )
        };

        assert_eq!(scale(DamageType::OnlyHealth), 1.0); // 无能量加成
        assert_eq!(scale(DamageType::OnlyShieldDefence), 1.0);
        assert_eq!(scale(DamageType::OnlyShieldSubstitute), 1.0);
        assert_eq!(scale(DamageType::OnlyShieldArcane), 1.0);
        // 对照:剪切有能量加成 (1*1) * (1+50/100) = 1.5
        assert_eq!(scale(DamageType::PhysicsShears), 1.5);
    }

    // endregion
}

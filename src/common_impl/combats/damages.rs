use crate::{
    base_lib::{
        cores::unify_types::FixedName,
        eff_attr_prop::{effects::Effect, prop_alter_eff::PropAlterEffectType},
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

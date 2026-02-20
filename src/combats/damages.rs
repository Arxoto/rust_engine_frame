use crate::{
    combats::{
        combat_additions::CombatAdditionAttr, combat_inherents::CombatInherentAttr,
        combat_units::CombatUnit,
    },
    cores::unify_type::FixedName,
};

// 文档注释中允许未使用的导入
#[allow(unused_imports)]
use crate::combats::combat_units::CombatHealthShield;

#[derive(Debug)]
pub enum DamageType {
    /// 真实伤害（因果论）
    KarmaTruth,
    /// 物理剪切 尖锐
    PhysicsShear,
    /// 物理冲击 沉重
    PhysicsImpact,
    /// 魔法奥术
    MagickaArcane,

    // 破盾特攻
    /// 防护破盾
    BrokeShieldDefence,
    /// 奥术破盾
    BrokeShieldArcane,
}

impl DamageType {
    /// 是否造成伤害（破盾类型不算造成伤害）
    pub fn is_hurt(&self) -> bool {
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

#[derive(Debug)]
pub struct DamageInfo {
    /// 伤害类型为“伤害”时表示是否致死，伤害类型为破盾时表示是否成功击穿防御
    pub broken: bool, // pub-external
    /// 造成的伤害（致死时伤害可能大于实际扣血值）
    pub damage: f64, // pub-external
}

impl DamageInfo {
    /// 是否由某一伤害类型造成的角色死亡
    pub fn is_dead(&self, damage_type: &DamageType) -> bool {
        self.broken && damage_type.is_hurt()
    }
}

/// # 如何衡量伤害公式是否平衡
///
/// - 单次伤害 应该与 受伤上限 成正比
/// - 伤害公式中各个属性的根源属性应该合理分配，避免某一属性影响力过大
///
/// ## 受伤上限
///
/// 受伤上限 本质即 生命值和护盾值 [`CombatHealthShield`]
///
/// - 生命值 [`CombatHealthShield::health`]
///   - 与 气力 [`CombatInherentAttr::strength`] 【呈正比】 公式 [`NumericalBalancer::calc_health_max`]
/// - 替身护盾 [`CombatHealthShield::shield_substitute`]
///   - 应该受某一属性的限制以防止数值膨胀 公式 todo
/// - 防护护盾 [`CombatHealthShield::shield_defence`]
///   - 与 盔甲坚韧 [`CombatAdditionAttr::armor_hard`] 【呈正比】 公式 [`NumericalBalancer::calc_defence_shield`]
///   - 与 气力 [`CombatInherentAttr::strength`] 【呈正比】 公式 todo 盔甲坚韧受气力影响
/// - 奥术护盾 [`CombatHealthShield::shield_arcane`]
///   - 应该受某一属性的限制以防止数值膨胀 公式 todo
///
/// 不同 伤害类型 [`DamageType`] 对应的 受伤上限 见 [`CombatHealthShield::hurt_external`]
///
/// - 真实伤害 [`DamageType::KarmaTruth`]
///   - 应与 生命值 【呈正比】
///   - 应与 气力 【呈正比】
/// - 物理剪切 [`DamageType::PhysicsShear`]
///   - 应与 生命值 + 替身 + 防护 【呈正比】
///   - todo
/// - todo
///
/// ## 单次伤害
///
/// 不同 伤害类型 [`DamageType`] 对应的 伤害公式 见 [`NumericalBalancer::calc_damage_scale`]
///
/// - 真实伤害 [`DamageType::KarmaTruth`]
///   - 为招式固有属性，与角色收获相关，使用 气力 代替
///   - 与 气力 [`CombatInherentAttr::strength`] 【呈正比】
/// - 物理剪切 [`DamageType::PhysicsShear`]
///   - 与 气力 [`CombatInherentAttr::strength`] * 武器锋利度 [`CombatAdditionAttr::weapon_sharp`] 【呈正比】
///   - 武器锋利度 为武器固有属性，与角色收获相关，使用 气力 代替
///   - 与 气力 [`CombatInherentAttr::strength`] ^2 【呈正比】
/// - todo
///
/// 单次伤害 与 受伤上限 数值平衡分析
///
/// - 真实伤害 [`DamageType::KarmaTruth`]
///   - 单次伤害 与 受伤上限 均与 气力 【呈正比】 符合平衡要求
/// - todo
pub struct NumericalBalancer;

impl NumericalBalancer {
    /// 伤害公式
    pub fn calc_damage_scale<S: FixedName>(
        damage_type: &DamageType,
        source_combat: &CombatUnit<S>,
        target_combat: &CombatUnit<S>,
    ) -> f64 {
        let damage_scale = match damage_type {
            DamageType::KarmaTruth => 1.0,
            DamageType::PhysicsShear | DamageType::BrokeShieldDefence => {
                source_combat.inherent_attr.strength.get_current()
                    * source_combat.addition_attr.weapon_sharp.get_current()
            }
            DamageType::PhysicsImpact => {
                (source_combat.inherent_attr.strength.get_current()
                    + source_combat.addition_attr.weapon_mass.get_current())
                    / target_combat.addition_attr.armor_soft.get_current()
            }
            DamageType::MagickaArcane | DamageType::BrokeShieldArcane => {
                source_combat.inherent_attr.belief.get_current()
            }
        };

        // 能量越高伤害越高 不使用双方能量差是为了防止在高能量状态下，小怪低能量形成的碾压，导致堆怪没威胁
        let base_scale = 1.0 + source_combat.magicka.get_current() / Self::get_default_prop_value();
        let base_scale = base_scale.max(0.0);

        damage_scale * base_scale
    }

    pub const fn get_default_prop_value() -> f64 {
        100.0
    }

    /// 血量计算
    pub fn calc_health_max<S: FixedName>(
        health_base: f64,
        health_scale: f64,
        inherent_attr: &CombatInherentAttr<S>,
    ) -> f64 {
        health_base + health_scale * inherent_attr.strength.get_origin()
    }

    /// 原始能量计算
    pub fn calc_magicka_value<S: FixedName>(
        magicka_base: f64,
        magicka_scale: f64,
        inherent_attr: &CombatInherentAttr<S>,
    ) -> f64 {
        magicka_base + magicka_scale * inherent_attr.belief.get_origin()
    }

    /// 能量能级计算
    pub fn calc_magicka_max<S: FixedName>(
        magicka_base: f64,
        magicka_scale: f64,
        inherent_attr: &CombatInherentAttr<S>,
        magicka_energy_level: &MagickaEnergyLevel,
    ) -> f64 {
        let magicka_value = Self::calc_magicka_value(magicka_base, magicka_scale, inherent_attr);
        magicka_energy_level.max_energy(magicka_value)
    }

    /// 护盾值计算
    pub fn calc_defence_shield<S: FixedName>(addition_attr: &CombatAdditionAttr<S>) -> f64 {
        addition_attr.armor_hard.get_current()
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

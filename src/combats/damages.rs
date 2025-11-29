use crate::{
    combats::{
        combat_additions::CombatAdditionAttr, combat_inherents::CombatInherentAttr,
        combat_units::CombatUnit,
    },
    cores::unify_type::FixedName,
};

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

pub struct NumericalBalancer;

impl NumericalBalancer {
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

        // 能量越高伤害越高 不使用能量差是因为防止后期堆怪没威胁
        let base_add =
            source_combat.magicka.get_current() / NumericalBalancer::get_default_prop_value();
        let base_scale = 1.0 + base_add.min(-1.0);

        damage_scale * base_scale
    }

    pub const fn get_default_prop_value() -> f64 {
        100.0
    }

    pub fn calc_health_max<S: FixedName>(
        health_base: f64,
        health_scale: f64,
        inherent_attr: &CombatInherentAttr<S>,
    ) -> f64 {
        health_base + health_scale * inherent_attr.belief.get_origin()
    }

    pub fn calc_magicka_max<S: FixedName>(
        magicka_base: f64,
        magicka_scale: f64,
        inherent_attr: &CombatInherentAttr<S>,
        magicka_energy_level: &MagickaEnergyLevel,
    ) -> f64 {
        let magicka_value = magicka_base + magicka_scale * inherent_attr.belief.get_origin();
        magicka_energy_level.max_energy(magicka_value)
    }

    pub fn calc_defence_shield<S: FixedName>(addition_attr: &CombatAdditionAttr<S>) -> f64 {
        addition_attr.armor_hard.get_current()
    }
}

pub struct MagickaEnergyLevel(f64, f64, f64);

impl MagickaEnergyLevel {
    pub fn new(l0: f64, l1: f64, l2: f64) -> MagickaEnergyLevel {
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

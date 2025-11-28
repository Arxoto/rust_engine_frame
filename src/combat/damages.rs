use crate::{
    attrs::dyn_prop_dur_effect::DynPropDurEffect,
    combat::{combat_additions::CombatAdditionAttr, combat_units::CombatUnit},
    cores::unify_type::FixedName,
    effects::duration_effect::EffectBuilder,
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

pub struct DamageSystem;

impl DamageSystem {
    pub fn calc_damage_scale<S: FixedName>(
        damage_type: &DamageType,
        source_combat: &CombatUnit<S>,
        target_combat: &CombatUnit<S>,
    ) -> f64 {
        match damage_type {
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
        }
    }

    pub fn gen_defence_shield<S: FixedName, T: Into<S>>(
        from_name: T,
        effect_name: T,
        addition_attr: &CombatAdditionAttr<S>,
    ) -> DynPropDurEffect<S> {
        let armor_hard = addition_attr.armor_hard.get_current();
        DynPropDurEffect::new_max_val(EffectBuilder::new_infinite(
            from_name,
            effect_name,
            armor_hard,
        ))
    }
}

pub struct EnergyLevel(f64, f64, f64);

impl EnergyLevel {
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

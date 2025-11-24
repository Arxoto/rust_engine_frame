use std::ops::Not;

use crate::{
    attrs::{dyn_prop::DynProp, dyn_prop_inst_effect::DynPropInstEffect},
    combat::{combat_units::CombatUnit, equipments::EquipUnit},
    cores::unify_type::FixedName,
    effects::{
        duration_effect::EffectBuilder,
        native_effect::{Effect, ProxyEffect},
    },
};

#[derive(Clone, Copy, Debug)]
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
    pub(crate) broken: bool,
    /// 造成的伤害（致死时伤害可能大于实际扣血值）
    pub(crate) damage: f64,
}

impl DamageInfo {
    /// 是否由某一伤害类型造成的角色死亡
    pub fn is_dead(&self, damage_type: &DamageType) -> bool {
        self.broken && damage_type.is_hurt()
    }
}

pub struct HealthShieldComponent<S: FixedName = String> {
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

impl<S: FixedName> HealthShieldComponent<S> {
    /// 真实伤害直接伤害血量 返回表示是否死亡
    pub fn hurt_karma_truth(&mut self, eff: DynPropInstEffect<S>) -> DamageInfo {
        let dyn_prop_alter_result = self.health.use_inst_effect(eff);
        let dead = self.health.alter_to_min(&dyn_prop_alter_result);
        DamageInfo {
            broken: dead,
            damage: dyn_prop_alter_result.value,
        }
    }

    /// 注意：返回值仅表示所有的属性均被清空，当传入的参数中没有血量时，不代表血量为空
    fn hurt_internal(mut eff: Effect<S>, props: &mut [&mut DynProp<S>]) -> DamageInfo {
        let mut dead_info = DamageInfo {
            broken: false,
            damage: eff.value,
        };

        for prop in props {
            let alter_result = prop.alter_current_value(&eff);
            let dead = prop.alter_to_min(&alter_result);

            dead_info.broken = dead;
            if dead_info.broken.not() {
                break;
            }
            eff.set_value(eff.get_value() - alter_result.delta);
        }

        dead_info
    }

    fn hurt_external(&mut self, damage_type: DamageType, eff: Effect<S>) -> DamageInfo {
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

    // todo from equip gen HealthAndShield
    // todo 依赖顺序排序 在combat模块里
    pub fn hurt(&mut self) -> DamageInfo {
        todo!()
    }
}

pub struct DamageComponent<S: FixedName> {
    pub(crate) damage_type: DamageType,
    pub(crate) damage_eff: DynPropInstEffect<S>,
}

impl<S: FixedName> DamageComponent<S> {
    pub fn gen_damage_eff(
        &self,
        source_name: &S,
        source_combat: &CombatUnit,
        source_equip: &EquipUnit,
        target_health: &HealthShieldComponent<S>,
        target_equip: &EquipUnit,
    ) -> Effect<S> {
        let base_prop = match self.damage_type {
            DamageType::KarmaTruth => &target_health.health,
            DamageType::PhysicsShear => &target_health.health,
            DamageType::PhysicsImpact => &target_health.health,
            DamageType::MagickaArcane => &target_health.health,
            DamageType::BrokeShieldDefence => &target_health.shield_defence,
            DamageType::BrokeShieldArcane => &target_health.shield_arcane,
        };
        let base_eff = self.damage_eff.clone().convert_real_effect(base_prop);

        let scale = match self.damage_type {
            DamageType::KarmaTruth => 1.0,
            DamageType::PhysicsShear | DamageType::BrokeShieldDefence => {
                source_combat.strength.get_current() * source_equip.weapon_sharp.get_current()
            }
            DamageType::PhysicsImpact => {
                (source_combat.strength.get_current() + source_equip.weapon_mass.get_current())
                    / target_equip.armor_soft.get_current()
            }
            DamageType::MagickaArcane | DamageType::BrokeShieldArcane => {
                source_combat.belief.get_current()
            }
        };

        let abs_value = scale * base_eff.value;
        EffectBuilder::new_instant(source_name.clone(), base_eff.effect_name, abs_value)
    }
}

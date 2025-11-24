use std::ops::Not;

use crate::{
    attrs::{dyn_prop::DynProp, dyn_prop_inst_effect::DynPropInstEffect},
    cores::unify_type::FixedName,
    effects::native_effect::{Effect, ProxyEffect},
};

pub enum DamageType {
    /// 真实伤害（因果论）
    KarmaTruth,
    /// 物理剪切 尖锐
    PhysicsShear,
    /// 物理冲击 沉重
    PhysicsImpact,
    /// 魔法奥术
    MagickaArcane,
}

#[derive(Debug)]
pub struct DeadInfo {
    /// 是否致死
    pub(crate) dead: bool,
    /// 造成的伤害（致死时伤害可能大于实际扣血值）
    pub(crate) damage: f64,
}

pub struct DamageComponent<S: FixedName = String> {
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

impl<S: FixedName> DamageComponent<S> {
    /// 真实伤害直接伤害血量 返回表示是否死亡
    pub fn hurt_karma_truth(&mut self, eff: DynPropInstEffect<S>) -> DeadInfo {
        let dyn_prop_alter_result = self.health.use_inst_effect(eff);
        let dead = self.health.alter_to_min(&dyn_prop_alter_result);
        DeadInfo {
            dead,
            damage: dyn_prop_alter_result.value,
        }
    }

    fn hurt_internal(mut eff: Effect<S>, props: &mut [&mut DynProp<S>]) -> DeadInfo {
        let mut dead_info = DeadInfo {
            dead: false,
            damage: eff.value,
        };

        for prop in props {
            let alter_result = prop.alter_current_value(&eff);
            let dead = prop.alter_to_min(&alter_result);

            dead_info.dead = dead;
            if dead_info.dead.not() {
                break;
            }
            eff.set_value(eff.get_value() - alter_result.delta);
        }

        dead_info
    }

    fn hurt_external(&mut self, damage_type: DamageType, eff: DynPropInstEffect<S>) -> DeadInfo {
        let eff = eff.convert_real_effect(&self.health);
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
        };
        Self::hurt_internal(eff, ll)
    }

    /// 破盾特攻
    fn broke_shield(&mut self, damage_type: DamageType, eff: DynPropInstEffect<S>) -> DeadInfo {
        let eff = eff.convert_real_effect(&self.health);
        let ll: &mut [&mut DynProp<S>] = match damage_type {
            DamageType::KarmaTruth => &mut [],
            DamageType::PhysicsShear => {
                &mut [&mut self.shield_defence, &mut self.shield_substitute]
            }
            DamageType::PhysicsImpact => &mut [&mut self.shield_substitute],
            DamageType::MagickaArcane => {
                &mut [&mut self.shield_arcane, &mut self.shield_substitute]
            }
        };
        Self::hurt_internal(eff, ll)
    }
}

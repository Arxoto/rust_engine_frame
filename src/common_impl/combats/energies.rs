use std::ops::{Deref, DerefMut};

use crate::base_lib::{cores::unify_types::FixedName, eff_attr::bounded_attr_effs::AttrAlterEff};

/// 能量消耗统一路径，因此无需自定义效果类型
#[derive(Debug)]
pub struct EnergyEffBuffer<S: FixedName>(Vec<AttrAlterEff<S>>);

impl<S: FixedName> Default for EnergyEffBuffer<S> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<S: FixedName> Deref for EnergyEffBuffer<S> {
    type Target = Vec<AttrAlterEff<S>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: FixedName> DerefMut for EnergyEffBuffer<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// 魔法能级划分
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

pub mod energy_system {
    use crate::{
        base_lib::{
            cores::unify_types::FixedName,
            eff_attr::{
                attr_layers::AttrLayerTypeIter, bounded_attr_effs::AttrAlterEff,
                modifier_collections::ModifiableAttr,
            },
        },
        common_impl::combats::{
            combat_inherents::Belief,
            combat_units::{EnergyAttrLayer, ExternalEnergy, Magicka, MagickaUpper},
            energies::{EnergyEffBuffer, MagickaEnergyLevel},
        },
    };

    /// 花费能量后允许的最低值(资源下限门槛,即时扣减用)
    const COST_FLOOR: f64 = 0.0;
    /// 能量的参考基线，用作伤害增幅计算
    const MAGICKA_BASELINE: f64 = 100.0;

    /// 花费能量(硬扣): 推入 buffer
    pub fn cost_magicka<S: FixedName>(buffer: &mut EnergyEffBuffer<S>, eff: AttrAlterEff<S>) {
        buffer.push(eff);
    }

    /// 结算能量花费，处理 [`cost_magicka`] 中推入 buffer 的效果
    pub fn apply_magicka_cost<S: FixedName>(
        magicka_upper: &MagickaUpper,
        magicka: &mut Magicka,
        ex_energy: &mut ExternalEnergy,
        eff_buffer: &mut EnergyEffBuffer<S>,
    ) {
        if eff_buffer.is_empty() {
            return;
        }

        let mut sum_val = 0.0;
        for eff in eff_buffer.drain(..) {
            let real_val = eff.calc_alter_val(&magicka.0, &magicka_upper.0);
            sum_val += real_val;
        }

        apply_magicka_val(magicka_upper, magicka, ex_energy, sum_val);
    }

    /// 尝试花费能量(软扣): 直接修改 pending ，能量不足则失败
    /// todo 扣除逻辑应该合并进层级属性
    pub fn try_cost_magicka<S: FixedName>(
        magicka_upper: &MagickaUpper,
        magicka: &mut Magicka,
        ex_energy: &mut ExternalEnergy,
        eff: AttrAlterEff<S>,
    ) -> bool {
        let bounded_attr = &mut magicka.0;
        let abs_val = eff.calc_alter_val(bounded_attr, &magicka_upper.0);
        bounded_attr.apply_eff_checked(COST_FLOOR, &magicka_upper.0, abs_val, COST_FLOOR)
    }

    fn apply_magicka_val_checked(
        magicka_upper: &MagickaUpper,
        magicka: &mut Magicka,
        ex_energy: &mut ExternalEnergy,
        mut val: f64,
    ) -> bool {
        let mut current_pending = 0.0;
        let energy_layers = AttrLayerTypeIter::from(EnergyAttrLayer::start_at());
        for energy_layer in energy_layers {
            let attr = match energy_layer {
                EnergyAttrLayer::Magicka => &mut magicka.0,
                EnergyAttrLayer::ExternalEnergy => &mut ex_energy.0,
            };

            current_pending += attr.get_pending_value();
        }

        todo!()
    }

    // todo 这里的 upper 和 lower 调用有问题，当轮到 ex_energy 时不应使用 magicka 的钳制
    fn apply_magicka_val(
        magicka_upper: &MagickaUpper,
        magicka: &mut Magicka,
        ex_energy: &mut ExternalEnergy,
        mut val: f64,
    ) {
        let energy_layers = AttrLayerTypeIter::from(EnergyAttrLayer::start_at());
        for energy_layer in energy_layers {
            let attr = match energy_layer {
                EnergyAttrLayer::Magicka => &mut magicka.0,
                EnergyAttrLayer::ExternalEnergy => &mut ex_energy.0,
            };

            let old_val = attr.get_pending_value();
            attr.apply_eff(val);
            attr.clamp_by(COST_FLOOR, &magicka_upper.0);
            let diff_val = attr.get_pending_value() - old_val;

            val -= diff_val;
        }
    }

    /// [`Belief`] 影响【原始能量】
    #[inline]
    pub(super) fn calc_magicka_value(
        magicka_base: f64,
        magicka_scale: f64,
        belief: &Belief,
    ) -> f64 {
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

    /// 以基线值做基准，计算弹性比例
    #[inline]
    pub fn calc_magicka_scale(magicka: &Magicka) -> f64 {
        magicka.0.get_snapshot_value() / MAGICKA_BASELINE
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        base_lib::eff_attr::stat_attrs::StatAttr,
        common_impl::combats::{
            combat_inherents::Belief,
            energies::energy_system::{calc_magicka_max, calc_magicka_value},
        },
    };

    use super::*;

    /// calc_magicka_value：Belief 影响原始能量，magicka_base + magicka_scale * belief.origin
    #[test]
    fn calc_magicka_value_scales_with_belief_origin() {
        let belief = Belief(StatAttr::new(10.0));
        assert_eq!(calc_magicka_value(50.0, 3.0, &belief), 80.0);
    }

    /// calc_magicka_max：先算原始能量，再按能级取对应层级上限
    #[test]
    fn calc_magicka_max_takes_energy_level() {
        let levels = MagickaEnergyLevel::new(100.0, 200.0, 300.0);
        // 原始能量 50 + 3*10 = 80 → 第一能级上限 100
        let belief = Belief(StatAttr::new(10.0));
        assert_eq!(calc_magicka_max(50.0, 3.0, &belief, &levels), 100.0);
        // 原始能量 50 + 3*50 = 200 → 第二能级上限 200
        let belief = Belief(StatAttr::new(50.0));
        assert_eq!(calc_magicka_max(50.0, 3.0, &belief, &levels), 200.0);
        // 原始能量 50 + 3*80 = 290 → 第三能级上限 300
        let belief = Belief(StatAttr::new(80.0));
        assert_eq!(calc_magicka_max(50.0, 3.0, &belief, &levels), 300.0);
    }
}

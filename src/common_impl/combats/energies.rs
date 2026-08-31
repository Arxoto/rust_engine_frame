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

#[cfg(test)]
mod tests {
    use crate::{
        base_lib::eff_attr::stat_attrs::StatAttr,
        common_impl::combats::{
            combat_inherents::Belief,
            energy_systems::{calc_magicka_max, calc_magicka_value},
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

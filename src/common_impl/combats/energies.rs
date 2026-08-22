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
        base_lib::{cores::unify_types::FixedName, eff_attr::attr_layers::AttrLayerTypeIter},
        common_impl::combats::{
            combat_units::{EnergyAttrLayer, ExternalEnergy, Magicka, MagickaUpper},
            energies::EnergyEffBuffer,
        },
    };

    const BOUNDED_ATTR_LOWER: f64 = 0.0;

    pub fn cost_magicka_energy<S: FixedName>(
        magicka_upper: &MagickaUpper,
        magicka: &mut Magicka,
        ex_energy: &mut ExternalEnergy,
        eff_buffer: &mut EnergyEffBuffer<S>,
    ) {
        if eff_buffer.is_empty() {
            return;
        }

        let mut sum_val = 0.0;
        for eff in eff_buffer.drain(0..) {
            let real_val = eff.calc_alter_val(&magicka.0, &magicka_upper.0);
            sum_val += real_val;
        }

        let energy_layers = AttrLayerTypeIter::from(EnergyAttrLayer::start_at());
        for energy_layer in energy_layers {
            let attr = match energy_layer {
                EnergyAttrLayer::Magicka => &mut magicka.0,
                EnergyAttrLayer::ExternalEnergy => &mut ex_energy.0,
            };

            let old_val = attr.get_pending_value();
            attr.apply_eff(sum_val);
            attr.clamp_by(BOUNDED_ATTR_LOWER, &magicka_upper.0);
            let diff_val = attr.get_pending_value() - old_val;

            sum_val -= diff_val;
        }
    }
}

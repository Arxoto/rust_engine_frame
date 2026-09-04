use crate::{
    base_lib::{
        cores::unify_types::FixedName,
        eff_attr::{
            bounded_attr_effs::AttrAlterEff, compound_attr_systems,
            modifier_collections::ModifiableAttr,
        },
    },
    common_impl::combats::{
        combat_inherents::Belief,
        energies::{
            EnergyAttrRef, EnergyBoundRef, EnergyEffBuffer, EnergyEffTargets, ExternalEnergy,
            ExternalEnergyUpper, Magicka, MagickaEnergyLevel, MagickaUpper,
        },
    },
};

/// 能量的参考基线，用作伤害增幅计算
const MAGICKA_BASELINE: f64 = 100.0;

/// 花费能量(硬扣): 推入 buffer
pub fn cost_magicka<S: FixedName>(
    magicka: &Magicka,
    magicka_upper: &MagickaUpper,
    buffer: &mut EnergyEffBuffer<S>,
    eff: AttrAlterEff<S>,
) {
    let real_val = eff.calc_alter_val(&magicka.0, &magicka_upper.0);
    buffer.push_delta_val(real_val);
}

/// 结算能量花费，处理 [`cost_magicka`] 中推入 buffer 的效果
pub fn apply_magicka_cost<S: FixedName>(
    magicka: &mut Magicka,
    ex_energy: &mut ExternalEnergy,
    magicka_upper: &MagickaUpper,
    ex_energy_upper: &ExternalEnergyUpper,
    eff_buffer: &mut EnergyEffBuffer<S>,
) {
    let delta_val = eff_buffer.take_delta_val();
    if delta_val == 0.0 {
        return;
    }

    // alter

    let mut attrs = EnergyAttrRef { magicka, ex_energy };
    let attr_bounds = EnergyBoundRef {
        magicka_upper,
        ex_energy_upper,
    };

    compound_attr_systems::apply_alter(
        &mut attrs,
        &attr_bounds,
        EnergyEffTargets::default(),
        delta_val,
    );
}

/// 尝试花费能量(软扣): 直接修改 pending ，能量不足则失败
pub fn try_cost_magicka<S: FixedName>(
    magicka: &mut Magicka,
    ex_energy: &mut ExternalEnergy,
    magicka_upper: &MagickaUpper,
    ex_energy_upper: &ExternalEnergyUpper,
    must_ge: f64,
    eff: AttrAlterEff<S>,
) -> bool {
    let delta_val = eff.calc_alter_val(&magicka.0, &magicka_upper.0);

    let mut attrs = EnergyAttrRef { magicka, ex_energy };
    let attr_bounds = EnergyBoundRef {
        magicka_upper,
        ex_energy_upper,
    };

    compound_attr_systems::apply_alter_safety(
        &mut attrs,
        &attr_bounds,
        EnergyEffTargets::default(),
        delta_val,
        must_ge,
    )
}

/// [`Belief`] 影响【原始能量】
#[inline]
fn calc_magicka_value(magicka_base: f64, magicka_scale: f64, belief: &Belief) -> f64 {
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

#[cfg(test)]
mod tests {
    use crate::base_lib::eff_attr::stat_attrs::StatAttr;

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

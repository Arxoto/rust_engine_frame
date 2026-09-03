use crate::{
    base_lib::{
        cores::unify_types::FixedName,
        eff_attr::{
            attr_layers::{AttrLayerEffTarget, AttrLayerEffTargetIter},
            bounded_attr_effs::AttrAlterEff,
            compound_attr_systems,
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

/// 花费能量后允许的最低值(资源下限门槛,即时扣减用)
const COST_FLOOR: f64 = 0.0;
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
    let mut eff = eff.take_eff();
    eff.set_effect_value(real_val);
    buffer.push(eff);
}

/// 结算能量花费，处理 [`cost_magicka`] 中推入 buffer 的效果
pub fn apply_magicka_cost<S: FixedName>(
    magicka: &mut Magicka,
    ex_energy: &mut ExternalEnergy,
    magicka_upper: &MagickaUpper,
    ex_energy_upper: &ExternalEnergyUpper,
    eff_buffer: &mut EnergyEffBuffer<S>,
) {
    if eff_buffer.is_empty() {
        return;
    }

    // merge

    let mut delta_val = 0.0;
    for eff in eff_buffer.drain(..) {
        delta_val += eff.get_effect_value();
    }

    // alter

    let mut attrs = EnergyAttrRef { magicka, ex_energy };
    let attr_bounds = EnergyBoundRef {
        magicka_upper,
        ex_energy_upper,
    };
    let energy_layers = AttrLayerEffTargetIter::from(EnergyEffTargets::default());

    compound_attr_systems::apply_alter(&mut attrs, &attr_bounds, energy_layers, delta_val);
}

/// 尝试花费能量(软扣): 直接修改 pending ，能量不足则失败
pub fn try_cost_magicka<S: FixedName>(
    magicka: &mut Magicka,
    ex_energy: &mut ExternalEnergy,
    magicka_upper: &MagickaUpper,
    ex_energy_upper: &ExternalEnergyUpper,
    eff: AttrAlterEff<S>,
) -> bool {
    let delta_val = eff.calc_alter_val(&magicka.0, &magicka_upper.0);

    let mut attrs = EnergyAttrRef { magicka, ex_energy };
    let attr_bounds = EnergyBoundRef {
        magicka_upper,
        ex_energy_upper,
    };
    let energy_layers = AttrLayerEffTargetIter::from(EnergyEffTargets::default());

    let target_layer = EnergyEffTargets::default().stop_at();

    compound_attr_systems::apply_alter_safety(
        &mut attrs,
        &attr_bounds,
        energy_layers,
        delta_val,
        target_layer,
        COST_FLOOR,
    )
}

/// [`Belief`] 影响【原始能量】
#[inline]
pub(super) fn calc_magicka_value(magicka_base: f64, magicka_scale: f64, belief: &Belief) -> f64 {
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

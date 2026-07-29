//! prop 属性的上下限效果，表现为血量上下限

use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr_prop::{
        attr_eff::{AttrEffect, AttrEffectType},
        effects::Effect,
        upsert_container::Upsert,
    },
};

/// prop 属性上下限效果，本质为 [`AttrEffect`] ，但只支持的维度有限制
pub fn new_val_eff<S: FixedName, Timer: Upsert>(
    effect: Effect<S>,
    duration: Timer,
) -> AttrEffect<S, Timer> {
    AttrEffect::new(AttrEffectType::BasicAdd, effect, duration)
}

/// prop 属性上下限效果，本质为 [`AttrEffect`] ，但只支持的维度有限制
///
/// 若想在修改上限的同时修改实际值，那么需要同时生成【修改上限】的效果和【修改实际值】的效果
///
/// 为了保证两者修改效果一致，限制修改维度只能基于基础值修改（不会被放大缩小产生偏差）
pub fn new_per_eff<S: FixedName, Timer: Upsert>(
    effect: Effect<S>,
    duration: Timer,
) -> AttrEffect<S, Timer> {
    AttrEffect::new(AttrEffectType::BasicPer, effect, duration)
}

//! prop 属性的实际值效果，表现为扣血蓝耗等

use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr_prop::effects::{Effect, EffectMean, EffectMeaning},
};

/// prop 属性效果 一般用作扣血蓝耗等
///
/// 考虑到伤害公式的计算，这里只支持绝对值
#[derive(Clone, Debug)]
pub struct PropEffect<S: FixedName> {
    eff: Effect<S>,
}

impl<S: FixedName> PropEffect<S> {
    pub fn new(eff: Effect<S>) -> Self {
        Self { eff }
    }

    pub fn get_from_name(&self) -> &S {
        self.eff.get_from_name()
    }

    pub fn get_effect_name(&self) -> &S {
        self.eff.get_effect_name()
    }

    pub fn get_effect_value(&self) -> f64 {
        self.eff.get_effect_value()
    }
}

impl<S: FixedName> EffectMeaning for PropEffect<S> {
    fn which_nature(&self) -> EffectMean {
        EffectMean::which_nature(self.eff.get_effect_value(), 0.0)
    }
}

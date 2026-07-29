//! prop 属性的实际值效果，表现为扣血蓝耗等

use crate::base_lib::{cores::unify_types::FixedName, eff_attr_prop::effects::Effect};

/// prop 属性效果的类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PropEffectType {
    /// 直接修改当前值
    Val,
    /// 根据当前值的百分比 修改当前值
    CurPer,
    /// 根据最大值的百分比 修改当前值
    MaxPer,
}

#[derive(Clone, Debug)]
pub struct PropEffectContext {
    pub current_value: f64,
    pub max_value: f64,
}

/// prop 属性效果 一般用作扣血蓝耗等
#[derive(Clone, Debug)]
pub struct PropEffect<S: FixedName> {
    eff_type: PropEffectType,
    effect: Effect<S>,
}

impl<S: FixedName> PropEffect<S> {
    pub fn new(eff_type: PropEffectType, effect: Effect<S>) -> Self {
        Self { eff_type, effect }
    }

    /// 将瞬时效果转化成绝对值（如恢复最大值的百分比，转换成绝对值）
    pub(super) fn to_abs_eff(mut self, ctx: &PropEffectContext) -> Effect<S> {
        match self.eff_type {
            PropEffectType::Val => self.effect,
            PropEffectType::CurPer => {
                let abs_value = self.effect.get_effect_value() * ctx.current_value;
                self.effect.set_eff_val(abs_value);
                self.effect
            }
            PropEffectType::MaxPer => {
                let abs_value = self.effect.get_effect_value() * ctx.max_value;
                self.effect.set_eff_val(abs_value);
                self.effect
            }
        }
    }
}

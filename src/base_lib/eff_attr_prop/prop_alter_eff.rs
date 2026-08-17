//! 对任意 Prop 的通用修改描述与直接应用
//!
//! 用于单次瞬时生效的修改(如花费资源、削减平衡),与 [`super::effects::Effect`]
//! 的"负值 = 减益"语义一致:减益用负 `effect_value`,百分比按目标 Prop 自身折算。

use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr_prop::{effects::Effect, props::Prop},
};

/// 对 Prop 的修改计算方式
#[derive(Debug, Clone, Copy)]
pub enum PropAlterEffectType {
    /// 绝对值修改
    Val,
    /// 根据当前值的百分比
    CurPer,
    /// 根据最大值的百分比
    MaxPer,
}

impl PropAlterEffectType {
    /// 根据效果类型计算绝对值
    ///
    /// 为了内聚 逻辑必须在这里实现 因此需要传入 [`Prop`]
    pub fn calc_alter_val(&self, eff_val: f64, prop: &Prop) -> f64 {
        match self {
            Self::Val => eff_val,
            Self::CurPer => eff_val * prop.get_current(),
            Self::MaxPer => eff_val * prop.get_max(),
        }
    }
}

/// 对 Prop 的修改描述:计算方式 + 效果
#[derive(Debug, Clone)]
pub struct PropAlterEffect<S: FixedName> {
    eff_type: PropAlterEffectType,
    eff: Effect<S>,
}

impl<S: FixedName> PropAlterEffect<S> {
    /// 构造修改效果
    pub fn new(eff_type: PropAlterEffectType, eff: Effect<S>) -> Self {
        Self { eff_type, eff }
    }

    /// 计算 [`PropAlterEffect`] 为绝对值（参照目标对应的 [`Prop`] ）
    pub fn calc_alter_val(&self, prop: &Prop) -> f64 {
        let eff_val = self.eff.get_effect_value();
        self.eff_type.calc_alter_val(eff_val, prop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base_lib::eff_attr_prop::props::Prop;

    /// Val:绝对值直接作用,返回实际生效值
    #[test]
    fn apply_val_uses_absolute_value() {
        let mut prop = Prop::new(30.0, 100.0, 0.0);
        let eff =
            PropAlterEffect::new(PropAlterEffectType::Val, Effect::new("from", "cost", -20.0));
        let abs_val = eff.calc_alter_val(&prop);
        let res = prop.apply_eff(abs_val);
        assert_eq!(prop.get_current(), 10.0);
        assert_eq!(res.real_eff_val, -20.0);
    }

    /// CurPer:按当前值百分比折算(参照目标 Prop 自身)
    #[test]
    fn apply_cur_per_scales_on_current() {
        let mut prop = Prop::new(50.0, 100.0, 0.0);
        // -0.5 * 50 = -25
        let eff = PropAlterEffect::new(
            PropAlterEffectType::CurPer,
            Effect::new("from", "cut", -0.5),
        );
        let abs_val = eff.calc_alter_val(&prop);
        let res = prop.apply_eff(abs_val);
        assert_eq!(prop.get_current(), 25.0);
        assert_eq!(res.real_eff_val, -25.0);
    }

    /// MaxPer:按最大值百分比折算(参照目标 Prop 自身)
    #[test]
    fn apply_max_per_scales_on_max() {
        let mut prop = Prop::new(30.0, 100.0, 0.0);
        // -0.1 * 100 = -10
        let eff = PropAlterEffect::new(
            PropAlterEffectType::MaxPer,
            Effect::new("from", "cut", -0.1),
        );
        let abs_val = eff.calc_alter_val(&prop);
        let res = prop.apply_eff(abs_val);
        assert_eq!(prop.get_current(), 20.0);
        assert_eq!(res.real_eff_val, -10.0);
    }
}

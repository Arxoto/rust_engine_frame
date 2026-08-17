//! 对任意 Prop 的通用修改描述与修改值计算

use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr_prop::{
        effects::Effect,
        prop_bounds_eff::{PropBoundsEffect, PropBoundsEffectLogic, PropBoundsEffectType},
        props::Prop,
    },
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

/// 对 Prop 的修改效果: 计算方式 + 效果
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

    pub fn get_type(&self) -> PropAlterEffectType {
        self.eff_type
    }

    pub fn own_eff(self) -> Effect<S> {
        self.eff
    }

    /// 计算 [`PropAlterEffect`] 为绝对值（参照目标对应的 [`Prop`] ）
    pub fn calc_alter_val(&self, prop: &Prop) -> f64 {
        let eff_val = self.eff.get_effect_value();
        self.eff_type.calc_alter_val(eff_val, prop)
    }

    // todo add test
    /// 生成一致的 [`PropAlterEffect`] [`PropBoundsEffect`] 效果
    ///
    /// 若是针对下限生效的效果，则不生成 [`PropAlterEffect`] ，应该通过 [`Prop::apply_bounds`] 自动应用影响
    pub fn gen_alter_bounds_eff<Timer>(
        prop: &Prop,
        eff_type: PropBoundsEffectType,
        effect: Effect<S>,
        duration: Timer,
    ) -> (Option<PropAlterEffect<S>>, PropBoundsEffect<S, Timer>) {
        match eff_type {
            PropBoundsEffectType::UpperAdd => {}
            PropBoundsEffectType::UpperPer => {}
            PropBoundsEffectType::LowerAdd => {
                return (None, PropBoundsEffect::new(eff_type, effect, duration));
            }
        };

        let eff_logic: PropBoundsEffectLogic = eff_type.into();
        let (alter_eff, bounds_eff) =
            Self::gen_alter_bounds_eff_for_upper(prop, eff_logic, effect, duration);
        (Some(alter_eff), bounds_eff)
    }

    /// 始终生成针对于上限的 [`PropAlterEffect`] [`PropBoundsEffect`] 效果
    pub fn gen_alter_bounds_eff_for_upper<Timer>(
        prop: &Prop,
        eff_logic: PropBoundsEffectLogic,
        mut effect: Effect<S>,
        duration: Timer,
    ) -> (PropAlterEffect<S>, PropBoundsEffect<S, Timer>) {
        let eff_val = match eff_logic {
            PropBoundsEffectLogic::BasicAdd => effect.get_effect_value(),
            PropBoundsEffectLogic::BasicPer => effect.get_effect_value() * prop.get_max_origin(),
        };
        effect.set_effect_value(eff_val);

        // 所有的效果都以 BasicAdd 生效
        Self::gen_alter_bounds_eff_by_add_val(effect, duration)
    }

    /// 以 BasicAdd 绝对值 的方式，构建一致的 [`PropAlterEffect`] [`PropBoundsEffect`] 效果
    pub fn gen_alter_bounds_eff_by_add_val<Timer>(
        effect: Effect<S>,
        duration: Timer,
    ) -> (Self, PropBoundsEffect<S, Timer>) {
        let eff_type = PropAlterEffectType::Val;
        let prop_alter_effect = Self::new(eff_type, effect.clone());

        let eff_type = PropBoundsEffectType::UpperAdd;
        let prop_bounds_effect = PropBoundsEffect::new(eff_type, effect, duration);

        (prop_alter_effect, prop_bounds_effect)
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

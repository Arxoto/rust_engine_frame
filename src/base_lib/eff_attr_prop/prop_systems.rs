//! Prop 相关的 System 层
//!
//! 与 [`super::attr_systems`] 对称:attr_systems 刷新 `Attr` 的脏属性,
//! 本模块刷新 `Prop` 的上下限。ECS 每帧由这里依脏标签统一处理,
//! 业务代码(如装载护盾)不直接调用 `Prop::refresh_bounds`。

use crate::base_lib::{
    cores::{timers::static_timer::StaticTimer, unify_types::FixedName},
    eff_attr_prop::{
        prop_bounds_eff::PropBoundsEffect, props::Prop, upsert_container::UpsertContainer,
    },
};

/// 刷新脏上限:依容器的脏标签重算上下限并钳制当前值
///
/// 与 [`super::attr_systems::try_refresh_dirty_attr`] 对称——容器被修改
/// (upsert/delete)后置脏,本函数在脏时调用 [`Prop::refresh_bounds`] +
/// [`Prop::apply_bounds`],将上下限与当前值对齐到效果集合;不脏则不动。
pub fn try_refresh_dirty_prop_bounds<S: FixedName>(
    prop: &mut Prop,
    effs: &mut UpsertContainer<PropBoundsEffect<S, StaticTimer>>,
) {
    if effs.is_changed() {
        effs.reset_changed_flag();
        prop.refresh_bounds(effs.iter_ele());
        prop.apply_bounds();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base_lib::{
        cores::timers::static_timer::StaticTimer,
        eff_attr_prop::{
            effects::Effect,
            prop_bounds_eff::{PropBoundsEffect, PropBoundsEffectType},
        },
    };

    /// 容器置脏后刷新:上限被效果抬高,当前值随之钳制
    #[test]
    fn refresh_dirty_prop_bounds_applies_upper_effect() {
        let mut prop = Prop::new(100.0, 100.0, 0.0);
        let mut effs: UpsertContainer<PropBoundsEffect<String, StaticTimer>> =
            UpsertContainer::default();

        // upsert 一个提高上限的效果 → 容器置脏
        effs.upsert_replace(PropBoundsEffect::new(
            PropBoundsEffectType::UpperAdd,
            Effect::new_form("buff", "max_hp", 50.0),
            StaticTimer::inf(),
        ));
        assert!(effs.is_changed());

        try_refresh_dirty_prop_bounds(&mut prop, &mut effs);
        assert_eq!(prop.get_max(), 150.0);
        assert_eq!(prop.get_current(), 100.0); // 未超上限,不钳制

        // 刷新后脏标签复位:不再动作
        try_refresh_dirty_prop_bounds(&mut prop, &mut effs);
        assert_eq!(prop.get_max(), 150.0);
    }

    /// 当前值超新上限时,刷新后被钳制回上限
    #[test]
    fn refresh_dirty_prop_bounds_clamps_over_max() {
        let mut prop = Prop::new(120.0, 100.0, 0.0); // 当前值已超上限(异常态)
        let mut effs: UpsertContainer<PropBoundsEffect<String, StaticTimer>> =
            UpsertContainer::default();

        // 上限被降到 80
        effs.upsert_replace(PropBoundsEffect::new(
            PropBoundsEffectType::UpperAdd,
            Effect::new_form("debuff", "lower_max", -20.0),
            StaticTimer::inf(),
        ));

        try_refresh_dirty_prop_bounds(&mut prop, &mut effs);
        assert_eq!(prop.get_max(), 80.0);
        assert_eq!(prop.get_current(), 80.0); // 钳制回上限
    }
}

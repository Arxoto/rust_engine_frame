use crate::{
    attrs::{
        dyn_prop::DynProp,
        dyn_prop_inst_effect::{DynPropInstEffect, DynPropInstEffectType},
    },
    cores::unify_type::FixedName,
    effects::{
        duration_effect::{DurationEffect, ProxyDurationEffect},
        native_duration::{Duration, ProxyDuration},
        native_effect::{Effect, ProxyEffect},
    },
};

/// prop属性周期效果的类型
#[derive(Clone, Copy)]
pub enum DynPropPeriodEffectType {
    /// 持续修改当前值
    CurVal,
    /// 持续百分比地增加当前值
    CurPer,
    /// 持续根据最大值的百分比修改当前值
    CurMaxPer,

    /// 使当前值逐渐逼近特定值 注意当效果值为负数时会不断远离
    ///
    /// 注意 由于效果容器底层为进行遍历时key不固
    CurValToVal(f64),
}

#[derive(Clone)]
pub struct DynPropPeriodEffect<S> {
    the_type: DynPropPeriodEffectType,
    effect: DurationEffect<S>,
}

impl<S> ProxyEffect<S> for DynPropPeriodEffect<S> {
    fn as_effect(&self) -> &Effect<S> {
        self.effect.as_effect()
    }

    fn as_mut_effect(&mut self) -> &mut Effect<S> {
        self.effect.as_mut_effect()
    }
}

impl<S> ProxyDuration for DynPropPeriodEffect<S> {
    fn as_duration(&self) -> &Duration {
        self.effect.as_duration()
    }

    fn as_mut_duration(&mut self) -> &mut Duration {
        self.effect.as_mut_duration()
    }
}

impl<S: Clone> ProxyDurationEffect<S> for DynPropPeriodEffect<S> {}

impl<S> DynPropPeriodEffect<S>
where
    S: FixedName,
{
    /// 无限存在的效果
    pub fn new_infinite(
        the_type: DynPropPeriodEffectType,
        from_name: S,
        effect_name: S,
        value: f64,
        period_time: f64,
    ) -> Self {
        let mut eff = Self {
            the_type,
            effect: DurationEffect::new_infinite(from_name, effect_name, value),
        };
        eff.set_period_time(period_time);
        eff
    }

    /// 持续一段时间的效果
    pub fn new_duration(
        the_type: DynPropPeriodEffectType,
        from_name: S,
        effect_name: S,
        value: f64,
        duration_time: f64,
        period_time: f64,
    ) -> Self {
        let mut eff = Self {
            the_type,
            effect: DurationEffect::new_duration(from_name, effect_name, value, duration_time),
        };
        eff.set_period_time(period_time);
        eff
    }

    pub fn new(
        the_type: DynPropPeriodEffectType,
        effect: DurationEffect<S>,
        period_time: f64,
    ) -> Self {
        let mut eff = Self { the_type, effect };
        eff.set_period_time(period_time);
        eff
    }

    /// 生效效果 仅属性类调用
    ///
    /// 当且仅当 **修改当前值** 时生效（依赖于 [`Self::put_effect_proxy`] 中的逻辑，仅这些类型时才放入自己的容器中）
    pub(crate) fn do_effect_alter_proxy(&self, prop: &mut DynProp<S>, periods: i64) {
        let eff_value = self.effect.get_value() * (self.effect.get_stack() as f64);
        for _ in 0..periods {
            let (the_type, value) = match self.the_type {
                DynPropPeriodEffectType::CurVal => (DynPropInstEffectType::CurVal, eff_value),
                DynPropPeriodEffectType::CurPer => (DynPropInstEffectType::CurPer, eff_value),
                DynPropPeriodEffectType::CurMaxPer => (DynPropInstEffectType::CurMaxPer, eff_value),
                DynPropPeriodEffectType::CurValToVal(to_val) => (
                    DynPropInstEffectType::CurVal,
                    move_toward_delta(prop.get_current(), to_val, eff_value),
                ),
            };
            let effect = DynPropInstEffect::new_instant(
                the_type,
                self.effect.get_from_name().clone(),
                self.effect.get_effect_name().clone(),
                value,
            );
            effect.do_effect_proxy(prop);
        }
    }
}

/// 从 `source` 向 `target` 移动（不会超过）
///
/// 移动步进 `speed` 正数接近 负数远离（恰好相等则减少）
///
/// 获得应该移动的距离
fn move_toward_delta(source: f64, target: f64, step: f64) -> f64 {
    let delta = target - source;

    if step > 0.0 {
        if delta > 0.0 {
            delta.min(step)
        } else if delta < 0.0 {
            delta.max(-step)
        } else {
            0.0
        }
    } else if step < 0.0 {
        if delta >= 0.0 { step } else { -step }
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_toward_near() {
        assert_eq!(move_toward_delta(150.0, 100.0, 10.0), -10.0);
        assert_eq!(move_toward_delta(50.0, 100.0, 10.0), 10.0);

        assert_eq!(move_toward_delta(150.0, 100.0, 100.0), -50.0);
        assert_eq!(move_toward_delta(50.0, 100.0, 100.0), 50.0);

        assert_eq!(move_toward_delta(50.0, 50.0, 100.0), 0.0);
    }

    #[test]
    fn move_toward_far() {
        assert_eq!(move_toward_delta(150.0, 100.0, -10.0), 10.0);
        assert_eq!(move_toward_delta(50.0, 100.0, -10.0), -10.0);

        assert_eq!(move_toward_delta(150.0, 100.0, -100.0), 100.0);
        assert_eq!(move_toward_delta(50.0, 100.0, -100.0), -100.0);

        assert_eq!(move_toward_delta(50.0, 50.0, -100.0), -100.0);
    }

    #[test]
    fn move_toward_zero() {
        assert_eq!(move_toward_delta(150.0, 100.0, 0.0), 0.0);
        assert_eq!(move_toward_delta(50.0, 100.0, 0.0), 0.0);
        assert_eq!(move_toward_delta(50.0, 50.0, 100.0), 0.0);
    }
}

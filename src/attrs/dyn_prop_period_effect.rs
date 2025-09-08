use crate::{
    attrs::{
        dyn_prop::DynProp,
        dyn_prop_inst_effect::{DynPropInstEffect, DynPropInstEffectType},
    },
    cores::unify_type::FixedName,
    effects::{
        duration_effect::{EffectBuilder, ProxyDurationEffect},
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

    /// 中性效果 使当前值逐渐逼近特定值 注意当效果值为负数时会不断远离
    CurValToVal(f64),
}

#[derive(Clone)]
pub struct DynPropPeriodEffect<S> {
    pub(crate) the_type: DynPropPeriodEffectType,
    pub(crate) effect: Effect<S>,
    pub(crate) duration: Duration,
}

impl<S> ProxyEffect<S> for DynPropPeriodEffect<S> {
    fn as_effect(&self) -> &Effect<S> {
        &self.effect
    }

    fn as_mut_effect(&mut self) -> &mut Effect<S> {
        &mut self.effect
    }

    fn which_nature(&self) -> crate::effects::native_effect::EffectNature {
        // 若类型为引力斥力 则始终呈现中性效果
        match self.the_type {
            DynPropPeriodEffectType::CurValToVal(_) => {
                crate::effects::native_effect::EffectNature::Neutral
            }
            _ => self.effect.which_nature(),
        }
    }
}

impl<S> ProxyDuration for DynPropPeriodEffect<S> {
    fn as_duration(&self) -> &Duration {
        &self.duration
    }

    fn as_mut_duration(&mut self) -> &mut Duration {
        &mut self.duration
    }
}

impl<S: Clone> ProxyDurationEffect<S> for DynPropPeriodEffect<S> {}

impl<S> DynPropPeriodEffect<S>
where
    S: FixedName,
{
    pub fn new(
        the_type: DynPropPeriodEffectType,
        (effect, mut duration): (Effect<S>, Duration),
        period_time: f64,
    ) -> Self {
        duration.period_time = period_time;
        Self {
            the_type,
            effect,
            duration,
        }
    }

    /// 基于一个周期效果（流血） 生成对应的瞬时效果（扣血）
    pub(crate) fn convert_prop_inst_effect(self, prop: &DynProp<S>) -> DynPropInstEffect<S> {
        let eff_value = self.effect.get_value() * (self.duration.get_stack() as f64);

        let (the_type, real_value) = match self.the_type {
            DynPropPeriodEffectType::CurVal => (DynPropInstEffectType::CurVal, eff_value),
            DynPropPeriodEffectType::CurPer => (DynPropInstEffectType::CurPer, eff_value),
            DynPropPeriodEffectType::CurMaxPer => (DynPropInstEffectType::CurMaxPer, eff_value),
            DynPropPeriodEffectType::CurValToVal(to_val) => (
                DynPropInstEffectType::CurVal,
                move_toward_delta(prop.get_current(), to_val, eff_value),
            ),
        };

        let Effect {
            from_name,
            effect_name,
            ..
        } = self.effect;

        DynPropInstEffect::new(
            the_type,
            EffectBuilder::new_instant(from_name, effect_name, real_value),
        )
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
    use crate::effects::native_effect::EffectNature;

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

    /// 提醒：每当增加类型时，判断其是否符合 [`DynAttrEffect::which_nature`]
    #[test]
    fn test_nature_tips() {
        let types = vec![
            DynPropPeriodEffectType::CurVal,
            DynPropPeriodEffectType::CurPer,
            DynPropPeriodEffectType::CurMaxPer,
            DynPropPeriodEffectType::CurValToVal(0.0),
        ];

        fn get_base_line(the_type: &DynPropPeriodEffectType) -> f64 {
            match the_type {
                DynPropPeriodEffectType::CurVal => 0.0,
                DynPropPeriodEffectType::CurPer => 0.0,
                DynPropPeriodEffectType::CurMaxPer => 0.0,
                DynPropPeriodEffectType::CurValToVal(_) => f64::MAX,
            }
        }

        for the_type in types {
            let value = get_base_line(&the_type);
            let eff: DynPropPeriodEffect<&str> = DynPropPeriodEffect::new(
                the_type,
                EffectBuilder::new_infinite("from_name", "effect_name", value),
                0.0,
            );
            assert!(matches!(eff.which_nature(), EffectNature::Neutral));
        }
    }
}

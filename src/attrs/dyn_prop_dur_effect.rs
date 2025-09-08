use crate::{
    attrs::{dyn_attr_effect::DynAttrEffect, dyn_prop::DynProp},
    cores::unify_type::FixedName,
    effects::{
        duration_effect::ProxyDurationEffect,
        native_duration::{Duration, ProxyDuration},
        native_effect::{Effect, ProxyEffect},
    },
};

/// prop属性持久效果的生效对象
#[derive(Clone, Copy)]
pub(crate) enum DynPropDurEffectTarget {
    /// 修改最大值
    ForMax,
    /// 修改最小值
    ForMin,
}

/// prop属性持久效果的类型
///
/// 若想修改最大值的同时修改当前值，同时赋予持久和瞬时效果即可；
/// 因为最大值的修改仅限于基础值，与其他效果互不影响，因此修改的值是绝对的。
#[derive(Clone, Copy)]
pub enum DynPropDurEffectType {
    /// 修改最大值
    MaxVal,
    /// 百分比修改最大值
    MaxPer,
    /// 修改最小值
    MinVal,
}

/// prop属性持久效果 包括直接作用于最大最小值的效果 还有特殊效果如持续流血回蓝等
#[derive(Clone)]
pub struct DynPropDurEffect<S> {
    pub(crate) the_type: DynPropDurEffectType,
    pub(crate) effect: Effect<S>,
    pub(crate) duration: Duration,
}

impl<S> ProxyEffect<S> for DynPropDurEffect<S> {
    fn as_effect(&self) -> &Effect<S> {
        &self.effect
    }

    fn as_mut_effect(&mut self) -> &mut Effect<S> {
        &mut self.effect
    }
}

impl<S> ProxyDuration for DynPropDurEffect<S> {
    fn as_duration(&self) -> &Duration {
        &self.duration
    }

    fn as_mut_duration(&mut self) -> &mut Duration {
        &mut self.duration
    }
}

impl<S: Clone> ProxyDurationEffect<S> for DynPropDurEffect<S> {}

impl<S: FixedName> DynPropDurEffect<S> {
    pub fn new(the_type: DynPropDurEffectType, (effect, duration): (Effect<S>, Duration)) -> Self {
        Self {
            the_type,
            effect,
            duration,
        }
    }

    /// 将该类型转换成针对属性的对应效果
    pub(crate) fn convert_attr_effect(self) -> (DynPropDurEffectTarget, DynAttrEffect<S>) {
        match self.the_type {
            DynPropDurEffectType::MaxVal => (
                DynPropDurEffectTarget::ForMax,
                DynAttrEffect::new_basic_add((self.effect, self.duration)),
            ),
            DynPropDurEffectType::MaxPer => (
                DynPropDurEffectTarget::ForMax,
                DynAttrEffect::new_basic_percent((self.effect, self.duration)),
            ),
            DynPropDurEffectType::MinVal => (
                DynPropDurEffectTarget::ForMin,
                DynAttrEffect::new_basic_add((self.effect, self.duration)),
            ),
        }
    }

    /// 基于一个持久效果（提升最大生命值） 生成对应的瞬时效果（加血）
    ///
    /// 注意 仅支持【最大值的增益】 否则应该基于上下界限去自动调整
    pub(crate) fn convert_real_effect_for_max_buff(self, prop: &DynProp<S>) -> Option<Effect<S>> {
        if !self.nature_is_buff() {
            return None;
        }
        match self.the_type {
            DynPropDurEffectType::MaxVal => Some(self.effect),
            DynPropDurEffectType::MaxPer => {
                let mut real_eff = self.effect;
                real_eff.value *= prop.get_current();
                Some(real_eff)
            }
            DynPropDurEffectType::MinVal => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::effects::{duration_effect::EffectBuilder, native_effect::EffectNature};

    use super::*;

    /// 提醒：每当增加类型时，判断其是否符合 [`DynAttrEffect::which_nature`]
    #[test]
    fn test_nature_tips() {
        let types = vec![
            DynPropDurEffectType::MaxVal,
            DynPropDurEffectType::MaxPer,
            DynPropDurEffectType::MinVal,
        ];

        fn get_base_line(the_type: &DynPropDurEffectType) -> f64 {
            match the_type {
                DynPropDurEffectType::MaxVal => 0.0,
                DynPropDurEffectType::MaxPer => 0.0,
                DynPropDurEffectType::MinVal => 0.0,
            }
        }

        for the_type in types {
            let value = get_base_line(&the_type);
            let eff: DynPropDurEffect<&str> = DynPropDurEffect::new(
                the_type,
                EffectBuilder::new_infinite("from_name", "effect_name", value),
            );
            assert!(matches!(eff.which_nature(), EffectNature::Neutral));
        }
    }
}

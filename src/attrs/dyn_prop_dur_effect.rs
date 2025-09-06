use crate::{
    attrs::{dyn_attr_effect::DynAttrEffect, dyn_prop::DynProp},
    cores::unify_type::FixedName,
    effects::{
        duration_effect::{DurationEffect, ProxyDurationEffect},
        native_duration::{Duration, ProxyDuration},
        native_effect::{Effect, ProxyEffect},
    },
};

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
    the_type: DynPropDurEffectType,
    effect: DurationEffect<S>,
}

impl<S> ProxyEffect<S> for DynPropDurEffect<S> {
    fn as_effect(&self) -> &Effect<S> {
        self.effect.as_effect()
    }

    fn as_mut_effect(&mut self) -> &mut Effect<S> {
        self.effect.as_mut_effect()
    }
}

impl<S> ProxyDuration for DynPropDurEffect<S> {
    fn as_duration(&self) -> &Duration {
        self.effect.as_duration()
    }

    fn as_mut_duration(&mut self) -> &mut Duration {
        self.effect.as_mut_duration()
    }
}

impl<S: Clone> ProxyDurationEffect<S> for DynPropDurEffect<S> {}

impl<S> DynPropDurEffect<S>
where
    S: FixedName,
{
    /// 无限存在的效果
    pub fn new_infinite(
        the_type: DynPropDurEffectType,
        from_name: S,
        effect_name: S,
        value: f64,
    ) -> Self {
        Self {
            the_type,
            effect: DurationEffect::new_infinite(from_name, effect_name, value),
        }
    }

    /// 持续一段时间的效果
    pub fn new_duration(
        the_type: DynPropDurEffectType,
        from_name: S,
        effect_name: S,
        value: f64,
        duration_time: f64,
    ) -> Self {
        Self {
            the_type,
            effect: DurationEffect::new_duration(from_name, effect_name, value, duration_time),
        }
    }

    pub fn new(the_type: DynPropDurEffectType, effect: DurationEffect<S>) -> Self {
        Self { the_type, effect }
    }

    /// 赋予效果 仅属性类调用
    pub(crate) fn put_effect_proxy(self, prop: &mut DynProp<S>) {
        match self.the_type {
            DynPropDurEffectType::MaxVal => {
                prop.put_max_attr_effect_proxy(DynAttrEffect::new_basic_add(self.effect))
            }
            DynPropDurEffectType::MaxPer => {
                prop.put_max_attr_effect_proxy(DynAttrEffect::new_basic_percent(self.effect))
            }
            DynPropDurEffectType::MinVal => {
                prop.put_min_attr_effect_proxy(DynAttrEffect::new_basic_add(self.effect))
            }
        }
    }

    // /// 赋予效果 并立即对当前值进行对应修改 仅属性类调用
    // ///
    // /// 当且仅当 **修改极值** 时生效（依赖于 [`Self::put_effect_proxy`] 中的逻辑，仅这些类型时才能一次性影响当前值）
    // pub(crate) fn put_and_use_effect_proxy(self, prop: &mut DynProp<S>) {
    //     match self.the_type {
    //         DynPropDurEffectType::MaxVal => {}
    //         DynPropDurEffectType::MaxPer => {}
    //         DynPropDurEffectType::MinVal => {}
    //         _ => return,
    //     }

    // }
}

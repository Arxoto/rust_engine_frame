use crate::{
    attrs::dyn_prop::DynProp,
    cores::unify_type::FixedName,
    effects::{
        instant_effect::InstantEffect,
        native_effect::{Effect, ProxyEffect},
    },
};

/// prop属性瞬时效果的类型
#[derive(Clone, Copy)]
pub enum DynPropInstEffectType {
    /// 直接修改当前值
    CurVal,
    /// 根据当前值的百分比修改当前值
    CurPer,
    /// 根据最大值的百分比修改当前值
    CurMaxPer,
}

/// prop属性瞬时效果 一般用作扣血蓝耗等
#[derive(Clone)]
pub struct DynPropInstEffect<S> {
    the_type: DynPropInstEffectType,
    effect: InstantEffect<S>,
}

impl<S> DynPropInstEffect<S>
where
    S: FixedName,
{
    pub fn new_instant<T: Into<S>>(
        the_type: DynPropInstEffectType,
        from_name: T,
        effect_name: T,
        value: f64,
    ) -> Self {
        Self {
            the_type,
            effect: InstantEffect::new_instant(from_name, effect_name, value),
        }
    }

    pub fn new(the_type: DynPropInstEffectType, effect: Effect<S>) -> Self {
        Self {
            the_type,
            effect: InstantEffect::new(effect),
        }
    }

    /// 生效效果 应仅由prop调用
    pub(crate) fn do_effect_proxy(self, prop: &mut DynProp<S>) -> f64 {
        match self.the_type {
            DynPropInstEffectType::CurVal => prop.alter_current_value_proxy(self.effect),
            DynPropInstEffectType::CurPer => {
                let v = self.effect.get_value() * prop.get_current();
                prop.alter_current_value_proxy(InstantEffect::new_instant(
                    self.effect.get_from_name().clone(),
                    self.effect.get_effect_name().clone(),
                    v,
                ))
            }
            DynPropInstEffectType::CurMaxPer => {
                let v = self.effect.get_value() * prop.get_max();
                prop.alter_current_value_proxy(InstantEffect::new_instant(
                    self.effect.get_from_name().clone(),
                    self.effect.get_effect_name().clone(),
                    v,
                ))
            }
        }
    }
}

impl<S> ProxyEffect<S> for DynPropInstEffect<S> {
    fn as_effect(&self) -> &crate::effects::native_effect::Effect<S> {
        self.effect.as_effect()
    }

    fn as_mut_effect(&mut self) -> &mut crate::effects::native_effect::Effect<S> {
        self.effect.as_mut_effect()
    }
}

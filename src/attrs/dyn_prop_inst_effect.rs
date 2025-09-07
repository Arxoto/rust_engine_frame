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

    /// 将瞬时效果转换成针对当前值的实际效果
    pub(crate) fn convert_real_inst_effect(self, prop: &DynProp<S>) -> InstantEffect<S> {
        match self.the_type {
            DynPropInstEffectType::CurVal => self.effect,
            DynPropInstEffectType::CurPer => {
                let v = self.effect.get_value() * prop.get_current();
                InstantEffect::new_instant(
                    self.effect.get_from_name().clone(),
                    self.effect.get_effect_name().clone(),
                    v,
                )
            }
            DynPropInstEffectType::CurMaxPer => {
                let v = self.effect.get_value() * prop.get_max();
                InstantEffect::new_instant(
                    self.effect.get_from_name().clone(),
                    self.effect.get_effect_name().clone(),
                    v,
                )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod tests {
        use crate::effects::native_effect::EffectNature;

        use super::*;

        /// 提醒：每当增加类型时，判断其是否符合 [`DynAttrEffect::which_nature`]
        #[test]
        fn test_nature_tips() {
            let types = vec![
                DynPropInstEffectType::CurVal,
                DynPropInstEffectType::CurPer,
                DynPropInstEffectType::CurMaxPer,
            ];

            fn get_base_line(the_type: &DynPropInstEffectType) -> f64 {
                match the_type {
                    DynPropInstEffectType::CurVal => 0.0,
                    DynPropInstEffectType::CurPer => 0.0,
                    DynPropInstEffectType::CurMaxPer => 0.0,
                }
            }

            for the_type in types {
                let value = get_base_line(&the_type);
                let eff: DynPropInstEffect<&str> =
                    DynPropInstEffect::new_instant(the_type, "from_name", "effect_name", value);
                assert!(matches!(eff.which_nature(), EffectNature::Neutral));
            }
        }
    }
}

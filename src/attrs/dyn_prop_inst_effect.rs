use crate::{
    attrs::dyn_prop::DynProp,
    cores::unify_type::FixedName,
    effects::native_effect::{Effect, ProxyEffect},
};

/// prop属性瞬时效果的类型
#[derive(Clone, Copy, Debug)]
pub enum DynPropInstEffectType {
    /// 直接修改当前值
    CurVal,
    /// 根据当前值的百分比修改当前值
    CurPer,
    /// 根据最大值的百分比修改当前值
    CurMaxPer,
}

/// prop属性瞬时效果 一般用作扣血蓝耗等
#[derive(Clone, Debug)]
pub struct DynPropInstEffect<S = String> {
    pub(crate) the_type: DynPropInstEffectType,
    pub(crate) effect: Effect<S>,
}

impl<S: FixedName> DynPropInstEffect<S> {
    pub fn new(the_type: DynPropInstEffectType, effect: Effect<S>) -> Self {
        Self { the_type, effect }
    }

    /// 将瞬时效果转换成针对当前值的实际效果
    pub(crate) fn convert_real_effect(self, prop: &DynProp<S>) -> Effect<S> {
        let DynPropInstEffect {
            the_type,
            mut effect,
        } = self;
        match the_type {
            DynPropInstEffectType::CurVal => effect,
            DynPropInstEffectType::CurPer => {
                effect.value *= prop.get_current();
                effect
            }
            DynPropInstEffectType::CurMaxPer => {
                effect.value *= prop.get_max();
                effect
            }
        }
    }
}

impl<S> ProxyEffect<S> for DynPropInstEffect<S> {
    fn as_effect(&self) -> &crate::effects::native_effect::Effect<S> {
        &self.effect
    }

    fn as_mut_effect(&mut self) -> &mut crate::effects::native_effect::Effect<S> {
        &mut self.effect
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod tests {
        use crate::effects::{duration_effect::EffectBuilder, native_effect::EffectNature};

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
                let eff: DynPropInstEffect = DynPropInstEffect::new(
                    the_type,
                    EffectBuilder::new_instant("from_name", "effect_name", value),
                );
                assert!(matches!(eff.which_nature(), EffectNature::Neutral));
            }
        }
    }
}

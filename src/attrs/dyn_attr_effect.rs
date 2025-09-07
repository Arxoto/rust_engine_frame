use crate::effects::{
    duration_effect::{DurationEffect, ProxyDurationEffect},
    native_duration::{Duration, ProxyDuration},
    native_effect::{Effect, ProxyEffect},
};

/// Attr属性效果的类型
#[derive(Clone, Copy)]
pub enum DynAttrEffectType {
    /// 基础加法（描述参考：基础伤害增加xx）
    BasicAdd,
    /// 最终乘法（描述参考：额外xx倍伤害），指数增长、谨慎使用
    FinalMulti,

    /// 基础百分比（描述参考：基础伤害提升xx%），可安全使用
    BasicPercent,
    /// 最终百分比（描述参考：伤害提升xx%），可安全使用
    FinalPercent,
}

/// Attr属性效果（周期性触发时堆叠效果）
#[derive(Clone)]
pub struct DynAttrEffect<S> {
    the_type: DynAttrEffectType,
    effect: DurationEffect<S>,
}

impl<S: Clone> DynAttrEffect<S> {
    pub fn new_infinite<T: Into<S>>(
        the_type: DynAttrEffectType,
        from_name: T,
        effect_name: T,
        value: f64,
    ) -> Self {
        Self {
            the_type,
            effect: DurationEffect::new_infinite(from_name, effect_name, value),
        }
    }

    pub fn new_duration<T: Into<S>>(
        the_type: DynAttrEffectType,
        from_name: T,
        effect_name: T,
        value: f64,
        duration_time: f64,
    ) -> Self {
        Self {
            the_type,
            effect: DurationEffect::new_duration(from_name, effect_name, value, duration_time),
        }
    }

    pub fn new(the_type: DynAttrEffectType, effect: DurationEffect<S>) -> Self {
        Self { the_type, effect }
    }
}

impl<S> ProxyEffect<S> for DynAttrEffect<S> {
    fn as_effect(&self) -> &Effect<S> {
        self.effect.as_effect()
    }

    fn as_mut_effect(&mut self) -> &mut Effect<S> {
        self.effect.as_mut_effect()
    }

    fn which_nature(&self) -> crate::effects::native_effect::EffectNature {
        match self.the_type {
            DynAttrEffectType::FinalMulti => {
                crate::effects::native_effect::EffectNature::which_nature(self.get_value(), 1.0)
            }
            _ => self.effect.which_nature(),
        }
    }
}

impl<S> ProxyDuration for DynAttrEffect<S> {
    fn as_duration(&self) -> &Duration {
        self.effect.as_duration()
    }

    fn as_mut_duration(&mut self) -> &mut Duration {
        self.effect.as_mut_duration()
    }
}

impl<S: Clone> ProxyDurationEffect<S> for DynAttrEffect<S> {}

// =================================================================================

/// Attr属性效果修改器
pub struct DynAttrModifier {
    basic_add: f64,
    basic_percent: f64,
    final_percent: f64,
    final_multi: f64,
}

impl Default for DynAttrModifier {
    fn default() -> Self {
        Self {
            basic_add: 0.0,
            basic_percent: 0.0,
            final_percent: 0.0,
            final_multi: 1.0,
        }
    }
}

impl DynAttrModifier {
    pub fn reduce<S>(&mut self, e: &DynAttrEffect<S>) {
        let eff = &e.effect;
        match e.the_type {
            DynAttrEffectType::BasicAdd => {
                self.basic_add += eff.get_value() * eff.get_stack() as f64
            }
            DynAttrEffectType::BasicPercent => {
                self.basic_percent += eff.get_value() * eff.get_stack() as f64
            }
            DynAttrEffectType::FinalPercent => {
                self.final_percent += eff.get_value() * eff.get_stack() as f64
            }
            DynAttrEffectType::FinalMulti => {
                self.final_multi *= eff
                    .get_value()
                    .powi(eff.get_stack().try_into().unwrap_or(0))
            }
        }
    }

    pub fn do_effect(&self, v: f64) -> f64 {
        (v + v * self.basic_percent + self.basic_add)
            * (1.0 + self.final_percent)
            * self.final_multi
    }
}

// =================================================================================

#[cfg(test)]
mod tests {
    use crate::effects::native_effect::EffectNature;

    use super::*;

    #[test]
    fn basic_add() {
        let mut modifier = DynAttrModifier::default();

        for _ in 0..5 {
            let eff: DynAttrEffect<&str> =
                DynAttrEffect::new_infinite(DynAttrEffectType::BasicAdd, "", "", 1.0);
            modifier.reduce(&eff);
        }

        assert_eq!(modifier.do_effect(0.0), 5.0);
    }

    #[test]
    fn basic_percent() {
        let mut modifier = DynAttrModifier::default();

        for _ in 0..5 {
            let eff: DynAttrEffect<&str> =
                DynAttrEffect::new_infinite(DynAttrEffectType::BasicPercent, "", "", 0.1);
            modifier.reduce(&eff);
        }

        assert_eq!(modifier.do_effect(1.0), 1.5);
    }

    #[test]
    fn add_percent() {
        let mut modifier = DynAttrModifier::default();

        for _ in 0..3 {
            let eff: DynAttrEffect<&str> =
                DynAttrEffect::new_infinite(DynAttrEffectType::BasicAdd, "", "", 1.0);
            modifier.reduce(&eff);
        }

        for _ in 0..3 {
            let eff: DynAttrEffect<&str> =
                DynAttrEffect::new_infinite(DynAttrEffectType::BasicPercent, "", "", 0.1);
            modifier.reduce(&eff);
        }

        assert_eq!(modifier.do_effect(2.0), 3.0 + 2.0 * 1.3);
    }

    #[test]
    fn final_percent() {
        let mut modifier = DynAttrModifier::default();

        for _ in 0..5 {
            let eff: DynAttrEffect<&str> =
                DynAttrEffect::new_infinite(DynAttrEffectType::BasicAdd, "", "", 1.0);
            modifier.reduce(&eff);
        }

        for _ in 0..5 {
            let eff: DynAttrEffect<&str> =
                DynAttrEffect::new_infinite(DynAttrEffectType::FinalPercent, "", "", 0.1);
            modifier.reduce(&eff);
        }

        assert_eq!(modifier.do_effect(0.0), 5.0 * 1.5);
    }

    #[test]
    fn final_multi() {
        let mut modifier = DynAttrModifier::default();

        for _ in 0..5 {
            let eff: DynAttrEffect<&str> =
                DynAttrEffect::new_infinite(DynAttrEffectType::BasicAdd, "", "", 1.0);
            modifier.reduce(&eff);
        }

        for _ in 0..5 {
            let eff: DynAttrEffect<&str> =
                DynAttrEffect::new_infinite(DynAttrEffectType::FinalMulti, "", "", 10.0);
            modifier.reduce(&eff);
        }

        assert_eq!(modifier.do_effect(0.0), 5.0 * 100000.0);
    }

    #[test]
    fn test_func() {
        let mut modifier = DynAttrModifier::default();

        for _ in 0..2 {
            let eff: DynAttrEffect<&str> =
                DynAttrEffect::new_infinite(DynAttrEffectType::BasicAdd, "", "", 1.0);
            modifier.reduce(&eff);
        }

        for _ in 0..3 {
            let eff: DynAttrEffect<&str> =
                DynAttrEffect::new_infinite(DynAttrEffectType::BasicPercent, "", "", 0.2);
            modifier.reduce(&eff);
        }

        for _ in 0..1 {
            let eff: DynAttrEffect<&str> =
                DynAttrEffect::new_infinite(DynAttrEffectType::FinalPercent, "", "", 0.1);
            modifier.reduce(&eff);
        }

        let eff: DynAttrEffect<&str> =
            DynAttrEffect::new_infinite(DynAttrEffectType::FinalMulti, "", "", 2.0);
        modifier.reduce(&eff);

        let eff: DynAttrEffect<&str> =
            DynAttrEffect::new_infinite(DynAttrEffectType::FinalMulti, "", "", 3.0);
        modifier.reduce(&eff);

        assert_eq!(modifier.do_effect(1.0), (2.0 + 1.0 * 1.6) * 1.1 * 2.0 * 3.0);
    }

    #[test]
    fn test_nature() {
        let mut dyn_attr_modifier = DynAttrModifier::default();
        let eff: DynAttrEffect<&str> =
            DynAttrEffect::new_infinite_basic_add("from_name", "effect_name", 1.0);
        dyn_attr_modifier.reduce(&eff);

        let vvv = 1.0;
        assert_eq!(
            dyn_attr_modifier.do_effect(vvv) > vvv,
            matches!(eff.which_nature(), EffectNature::Buff)
        );

        let eff: DynAttrEffect<&str> =
            DynAttrEffect::new_infinite_final_multi("from_name", "effect_name", 1.0);
        assert!(matches!(eff.which_nature(), EffectNature::Neutral));

        let eff: DynAttrEffect<&str> =
            DynAttrEffect::new_infinite_basic_percent("from_name", "effect_name", -0.1);
        assert!(matches!(eff.which_nature(), EffectNature::Debuff));

        let eff: DynAttrEffect<&str> =
            DynAttrEffect::new_infinite_basic_percent("from_name", "effect_name", 0.1);
        assert!(matches!(eff.which_nature(), EffectNature::Buff));
    }

    /// 提醒：每当增加类型时，判断其是否符合 [`DynAttrEffect::which_nature`]
    #[test]
    fn test_nature_tips() {
        let types = vec![
            DynAttrEffectType::BasicAdd,
            DynAttrEffectType::FinalMulti,
            DynAttrEffectType::BasicPercent,
            DynAttrEffectType::FinalPercent,
        ];

        fn get_base_line(the_type: &DynAttrEffectType) -> f64 {
            match the_type {
                DynAttrEffectType::BasicAdd => 0.0,
                DynAttrEffectType::FinalMulti => 1.0,
                DynAttrEffectType::BasicPercent => 0.0,
                DynAttrEffectType::FinalPercent => 0.0,
            }
        }

        for the_type in types {
            let value = get_base_line(&the_type);
            let eff: DynAttrEffect<&str> =
                DynAttrEffect::new_infinite(the_type, "from_name", "effect_name", value);
            assert!(matches!(eff.which_nature(), EffectNature::Neutral));
        }
    }
}

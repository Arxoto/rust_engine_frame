use crate::{
    attrs::{
        dyn_attr_effect::{DynAttrEffect, DynAttrModifier},
        effect_container::EffectContainer,
    },
    cores::unify_type::FixedName,
    effects::native_duration::ProxyDuration,
};

/// dynamic_attribute 属性 一般用作角色属性值 可被效果影响
#[derive(Debug)]
pub struct DynAttr<S: FixedName = String> {
    origin: f64,
    current: f64,
    effects: EffectContainer<S, DynAttrEffect<S>>,
}

impl<S: FixedName> DynAttr<S> {
    pub fn new(v: f64) -> Self {
        Self {
            origin: v,
            current: v,
            effects: EffectContainer::new(),
        }
    }

    pub fn new_with_current(v: f64, current: f64) -> Self {
        Self {
            origin: v,
            current: current,
            effects: EffectContainer::new(),
        }
    }

    pub fn get_origin(&self) -> f64 {
        self.origin
    }

    pub fn get_current(&self) -> f64 {
        self.current
    }

    /// 效果更新后刷新属性
    pub fn refresh_value(&mut self) {
        self.effects.refresh_order_keys();

        let mut dyn_attr_modifier = DynAttrModifier::default();
        for ele in self.effects.keys() {
            if let Some(eff) = self.effects.get_effect(&ele) {
                dyn_attr_modifier.reduce(eff);
            }
        }
        self.current = dyn_attr_modifier.do_effect(self.origin);
    }

    /// 装载效果
    ///
    /// 而后需 **手动调用** 刷新属性 [`Self::refresh_value`]
    pub fn put_or_stack_effect(&mut self, eff: DynAttrEffect<S>) {
        self.effects.put_or_stack_effect(eff);
    }

    /// 卸载效果
    ///
    /// 而后需 **手动调用** 刷新属性 [`Self::refresh_value`]
    pub fn del_effect(&mut self, s: &S) {
        self.effects.del_effect(s);
    }

    /// 无需手动刷新属性
    pub fn process_time(&mut self, delta: f64) {
        let mut changed = false;
        for ele in self.effects.keys() {
            let Some(eff) = self.effects.get_effect_mut(&ele) else {
                continue;
            };

            let periods = eff.process_period(delta);

            if eff.is_expired() {
                self.effects.del_effect(&ele);
                changed = true;
            } else if periods > 0 {
                // 周期性效果 堆叠层数
                eff.try_add_stack(periods);
                changed = true;
            }
        }

        if changed {
            self.refresh_value();
        }
    }
}

// =================================================================================

#[cfg(test)]
mod tests {
    use crate::effects::duration_effect::EffectBuilder;

    use super::*;

    #[test]
    fn refresh_value() {
        let mut attr: DynAttr = DynAttr::new(20.0);
        assert_eq!(attr.get_current(), 20.0);

        attr.put_or_stack_effect(DynAttrEffect::new_basic_add(EffectBuilder::new_infinite(
            "someone", "add", 70.0,
        )));
        attr.put_or_stack_effect(DynAttrEffect::new_basic_percent(
            EffectBuilder::new_infinite("someone", "per", 0.5),
        ));
        attr.put_or_stack_effect(DynAttrEffect::new_final_percent(
            EffectBuilder::new_infinite("someone", "perf", 0.5),
        ));
        attr.refresh_value();
        assert_eq!(attr.get_current(), 150.0);
    }

    #[test]
    fn duration() {
        let mut attr: DynAttr = DynAttr::new(100.0);
        attr.put_or_stack_effect(DynAttrEffect::new_basic_percent(
            EffectBuilder::new_duration("someone", "per", 0.5, 1.0),
        ));
        attr.refresh_value();
        assert_eq!(attr.get_current(), 150.0);

        // 未老化
        attr.process_time(0.4);
        assert_eq!(attr.get_current(), 150.0);

        // 达到老化时间 还原
        attr.process_time(0.7);
        assert_eq!(attr.get_current(), 100.0);
    }

    #[test]
    fn period_stack() {
        let mut attr: DynAttr = DynAttr::new(100.0);
        let mut eff =
            DynAttrEffect::new_basic_add(EffectBuilder::new_duration("someone", "eff", 50.0, 10.0));
        eff.set_wait_time(2.0);
        eff.set_period_time(1.0);
        eff.set_max_stack(0);
        attr.put_or_stack_effect(eff);
        attr.refresh_value();
        assert_eq!(attr.get_current(), 150.0);

        // 等待期间
        attr.process_time(1.0);
        assert_eq!(attr.get_current(), 150.0);

        // 等待结束 但未触发
        attr.process_time(1.5);
        assert_eq!(attr.get_current(), 150.0);

        // 第一次触发
        attr.process_time(1.0);
        assert_eq!(attr.get_current(), 200.0);

        // 第二三次触发
        attr.process_time(2.0);
        assert_eq!(attr.get_current(), 300.0);

        // 5.5s
        // 结束
        attr.process_time(4.5);
        assert_eq!(attr.get_current(), 100.0);
        assert_eq!(attr.effects.keys().len(), 0);

        attr.process_time(1.0);
        assert_eq!(attr.effects.keys().len(), 0);
    }
}

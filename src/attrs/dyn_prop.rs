use crate::{
    attrs::{
        dyn_attr::DynAttr, dyn_attr_effect::DynAttrEffect, dyn_prop_dur_effect::DynPropDurEffect,
        dyn_prop_inst_effect::DynPropInstEffect, dyn_prop_period_effect::DynPropPeriodEffect,
        effect_container::EffectContainer,
    },
    cores::unify_type::FixedName,
    effects::{
        duration_effect::ProxyDurationEffect, native_duration::ProxyDuration,
        native_effect::ProxyEffect,
    },
};

/// dynamic_property 属性 一般用作角色资源槽 可被效果影响
pub struct DynProp<S>
where
    S: FixedName,
{
    the_min: DynAttr<S>,
    the_max: DynAttr<S>,

    /// 当前值 可被实时改动
    ///
    /// 若效果为非幂等的 则适用于瞬时效果
    ///
    /// 若想实现“对血量的修改幅度增加10%”的效果 可理解为对“效果”生效的效果 需在该类添加独立的效果列表 每次修改时遍历
    current: f64,

    /// 周期效果 如流血等（影响最大最小值的效果在对应的内部attr里面）
    ///
    /// 需要注意 由于堆叠时直接覆盖来源 因此统计伤害时可能会不准确 根据游戏性自行调整
    effects: EffectContainer<S, DynPropPeriodEffect<S>>,
}

impl<S> DynProp<S>
where
    S: FixedName,
{
    pub fn new(v: f64, the_max: f64, the_min: f64) -> Self {
        Self {
            the_min: DynAttr::new(the_min),
            the_max: DynAttr::new(the_max),
            current: v,
            effects: EffectContainer::new(),
        }
    }

    pub fn new_with_max(v: f64) -> Self {
        Self::new(v, v, 0.0)
    }

    pub fn get_current(&self) -> f64 {
        self.current
    }

    pub fn get_max(&self) -> f64 {
        self.the_max.get_current()
    }

    pub fn get_min(&self) -> f64 {
        self.the_min.get_current()
    }

    fn fix_current(&mut self) {
        self.current = self.current.min(self.get_max());
        self.current = self.current.max(self.get_min());
    }

    /// 瞬时效果 返回对当前值的修改值 如造成伤害时处理护盾血量逻辑
    ///
    /// 无需手动修正属性值
    pub fn use_inst_effect(&mut self, e: DynPropInstEffect<S>) -> f64 {
        e.do_effect_proxy(self)
    }

    /// 对 max 或 min 装载了效果后 需要刷新以应用
    pub fn refresh_value(&mut self) {
        self.the_max.refresh_value();
        self.the_min.refresh_value();
        self.fix_current();
    }

    /// 持久效果 可外部调用
    ///
    /// 而后需 **手动调用** 刷新属性值 [`Self::refresh_value`]
    pub fn put_dur_effect(&mut self, e: DynPropDurEffect<S>) {
        e.put_effect_proxy(self);
    }

    /// 周期效果 可外部调用
    ///
    /// 而后需 **手动调用** 刷新属性值 [`Self::refresh_value`]
    pub fn put_period_effect(&mut self, e: DynPropPeriodEffect<S>) {
        self.effects.put_or_stack_effect(e);
    }

    /// 重启周期效果 仅刷新时间和来源 不影响值
    ///
    /// 而后需 **手动调用** 刷新属性值 [`Self::refresh_value`]
    pub fn restart_dur_effect<T: ProxyEffect<S>>(&mut self, e: &T) {
        if let Some(eff) = self.effects.get_effect_mut(e.get_effect_name()) {
            eff.refresh_with_name(e);
        }
    }

    /// 无需手动刷新属性值
    pub fn process_time(&mut self, delta: f64) {
        self.the_max.process_time(delta);
        self.the_min.process_time(delta);
        self.fix_current();

        for ele in self.effects.each_effect_names() {
            let Some(eff) = self.effects.get_effect_mut(&ele) else {
                continue;
            };

            let periods = eff.process_period(delta);
            let eff = eff.clone(); // 中断self的借用 之后不应该再对该类型做更改

            if eff.is_expired() {
                self.effects.del_effect(&ele);
            } else if periods > 0 {
                eff.do_effect_alter_proxy(self, periods);
            }
        }
    }

    // =================================================================================

    /// 赋予最大值效果 仅effect内部调用
    pub(crate) fn put_max_attr_effect_proxy(&mut self, e: DynAttrEffect<S>) {
        self.the_max.put_or_stack_effect(e);
    }

    /// 赋予最小值效果 仅effect内部调用
    pub(crate) fn put_min_attr_effect_proxy(&mut self, e: DynAttrEffect<S>) {
        self.the_min.put_or_stack_effect(e);
    }

    /// 如对血量直接造成伤害 return delta 用于处理护盾逻辑
    pub(crate) fn alter_current_value_proxy<T: ProxyEffect<S>>(&mut self, e: T) -> f64 {
        let the_old = self.get_current();
        self.current += e.get_value();
        self.fix_current();
        self.get_current() - the_old
    }
}

// =================================================================================

#[cfg(test)]
mod tests {
    use crate::effects::duration_effect::DurationEffect;

    use super::*;

    #[test]
    fn put_dur_effect_each_max_min() {
        let mut prop = DynProp::new_with_max(100.0);
        prop.put_dur_effect(DynPropDurEffect::new_max_per(DurationEffect::new_infinite(
            "from_name",
            "effect_name",
            -0.5,
        )));
        prop.refresh_value();
        assert_eq!(prop.get_max(), 50.0);
        assert_eq!(prop.get_current(), 50.0);

        let mut prop = DynProp::new_with_max(100.0);
        prop.put_dur_effect(DynPropDurEffect::new_max_per(DurationEffect::new_infinite(
            "from_name",
            "effect_name",
            0.5,
        )));
        prop.refresh_value();
        assert_eq!(prop.get_max(), 150.0);
        assert_eq!(prop.get_current(), 100.0);

        let mut prop = DynProp::new_with_max(100.0);
        prop.put_dur_effect(DynPropDurEffect::new_max_val(DurationEffect::new_infinite(
            "from_name",
            "effect_name",
            -50.0,
        )));
        prop.refresh_value();
        assert_eq!(prop.get_max(), 50.0);
        assert_eq!(prop.get_current(), 50.0);

        let mut prop = DynProp::new_with_max(100.0);
        prop.put_dur_effect(DynPropDurEffect::new_max_val(DurationEffect::new_infinite(
            "from_name",
            "effect_name",
            100.0,
        )));
        prop.put_dur_effect(DynPropDurEffect::new_min_val(DurationEffect::new_infinite(
            "from_name",
            "effect_name",
            150.0,
        )));
        prop.refresh_value();
        assert_eq!(prop.get_max(), 200.0);
        assert_eq!(prop.get_min(), 150.0);
        assert_eq!(prop.get_current(), 150.0);
    }

    #[test]
    fn use_inst_effect() {
        // should limit in max and min
        let mut prop = DynProp::new_with_max(100.0);
        assert_eq!(prop.get_current(), 100.0);

        prop.use_inst_effect(DynPropInstEffect::new_instant_cur_val(
            "someone",
            "effect_name",
            -50.0,
        ));
        assert_eq!(prop.get_current(), 50.0);

        prop.use_inst_effect(DynPropInstEffect::new_instant_cur_val(
            "someone",
            "effect_name",
            -50.0,
        ));
        assert_eq!(prop.get_current(), 0.0);

        prop.use_inst_effect(DynPropInstEffect::new_instant_cur_val(
            "someone",
            "effect_name",
            -50.0,
        ));
        assert_eq!(prop.get_current(), 0.0);

        prop.use_inst_effect(DynPropInstEffect::new_instant_cur_val(
            "someone",
            "effect_name",
            200.0,
        ));
        assert_eq!(prop.get_current(), 100.0);
    }

    #[test]
    fn put_period_effect_each_cur() {
        let mut prop = DynProp::new_with_max(100.0);
        let eff = DynPropPeriodEffect::new_cur_val(
            DurationEffect::new_infinite("from_name", "effect_name", -10.0),
            1.0,
        );
        prop.put_period_effect(eff);
        prop.refresh_value();
        assert_eq!(prop.get_current(), 100.0);
        prop.process_time(0.5);
        assert_eq!(prop.get_current(), 100.0);
        prop.process_time(1.0);
        assert_eq!(prop.get_current(), 90.0);
        prop.process_time(1.0);
        assert_eq!(prop.get_current(), 80.0);

        let mut prop = DynProp::new_with_max(100.0);
        let eff = DynPropPeriodEffect::new_cur_per(
            DurationEffect::new_infinite("from_name", "effect_name", -0.1),
            1.0,
        );
        prop.put_period_effect(eff);
        prop.refresh_value();
        assert_eq!(prop.get_current(), 100.0);
        prop.process_time(0.5);
        assert_eq!(prop.get_current(), 100.0);
        prop.process_time(1.0);
        assert_eq!(prop.get_current(), 90.0);
        prop.process_time(1.0);
        assert_eq!(prop.get_current(), 81.0);

        let mut prop = DynProp::new_with_max(100.0);
        let eff = DynPropPeriodEffect::new_cur_max_per(
            DurationEffect::new_infinite("from_name", "effect_name", -0.1),
            1.0,
        );
        prop.put_period_effect(eff);
        prop.refresh_value();
        assert_eq!(prop.get_current(), 100.0);
        prop.process_time(0.5);
        assert_eq!(prop.get_current(), 100.0);
        prop.process_time(1.0);
        assert_eq!(prop.get_current(), 90.0);
        prop.process_time(1.0);
        assert_eq!(prop.get_current(), 80.0);
    }

    #[test]
    fn put_period_effect_to_val_near() {
        let mut prop = DynProp::new_with_max(100.0);
        let mut eff = DynPropPeriodEffect::new_cur_val_to_val(
            DurationEffect::new_infinite("from_name", "effect_name", 9.0),
            50.0,
            1.0,
        );
        eff.set_max_stack(2);
        eff.set_stack(2);
        prop.put_period_effect(eff);
        prop.refresh_value();

        // 持续逼近
        prop.process_time(1.0);
        assert_eq!(prop.get_current(), 100.0 - 18.0 * 1.0);
        prop.process_time(1.0);
        assert_eq!(prop.get_current(), 100.0 - 18.0 * 2.0);
        // 达到目标值
        prop.process_time(1.0);
        assert_eq!(prop.get_current(), 50.0);

        // 另一个方向
        prop.current = 0.0;

        // 持续逼近
        prop.process_time(1.0);
        assert_eq!(prop.get_current(), 18.0 * 1.0);
        prop.process_time(1.0);
        assert_eq!(prop.get_current(), 18.0 * 2.0);
        // 达到目标值
        prop.process_time(1.0);
        assert_eq!(prop.get_current(), 50.0);
    }

    #[test]
    fn put_period_effect_to_val_far() {
        let mut prop = DynProp::new_with_max(100.0);
        prop.current = 50.0;
        let eff = DynPropPeriodEffect::new_cur_val_to_val(
            DurationEffect::new_infinite("from_name", "effect_name", -9.0),
            50.0,
            1.0,
        );
        prop.put_period_effect(eff);
        prop.refresh_value();

        // 持续逼近
        prop.process_time(1.0);
        assert_eq!(prop.get_current(), 50.0 - 9.0 * 1.0);
        prop.process_time(2.0);
        assert_eq!(prop.get_current(), 50.0 - 9.0 * 3.0);
        // 达到目标值
        prop.process_time(3.0);
        assert_eq!(prop.get_current(), 0.0);

        // 另一个方向
        prop.current = 51.0;

        // 持续逼近
        prop.process_time(1.0);
        assert_eq!(prop.get_current(), 51.0 + 9.0 * 1.0);
        prop.process_time(2.0);
        assert_eq!(prop.get_current(), 51.0 + 9.0 * 3.0);
        // 达到目标值
        prop.process_time(3.0);
        assert_eq!(prop.get_current(), 100.0);
    }

    // #[test]
    // fn put_period_effect_to_val_twice() {
    //     let mut prop = DynProp::new_with_max(100.0);
    //     let mut eff = DynPropPeriodEffect::new_cur_val_to_val(
    //         DurationEffect::new_infinite("from_name", "effect_name", 9.0),
    //         50.0,
    //         1.0,
    //     );
    // }

    #[test]
    fn refresh_for_wait_and_expired_time() {}

    #[test]
    fn refresh_for_stack() {}

    #[test]
    fn process_time() {}
}

use crate::{
    attrs::{
        dyn_attr::DynAttr,
        dyn_attr_effect::DynAttrEffect,
        dyn_prop_dur_effect::{DynPropDurEffect, DynPropDurEffectTarget},
        dyn_prop_inst_effect::DynPropInstEffect,
        dyn_prop_period_effect::DynPropPeriodEffect,
        effect_container::EffectContainer,
        event_prop::DynPropEvents,
    },
    cores::unify_type::FixedName,
    effects::{
        duration_effect::ProxyDurationEffect,
        native_duration::ProxyDuration,
        native_effect::{Effect, ProxyEffect},
    },
};

/// dynamic_property 属性 一般用作角色资源槽 可被效果影响
#[derive(Debug, Default)]
pub struct DynProp<S: FixedName = String> {
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
    period_effects: EffectContainer<S, DynPropPeriodEffect<S>>,
}

impl<S: FixedName> DynProp<S> {
    pub fn new(v: f64, the_max: f64, the_min: f64) -> Self {
        Self {
            the_min: DynAttr::new(the_min),
            the_max: DynAttr::new(the_max),
            current: v,
            period_effects: EffectContainer::new(),
        }
    }

    pub fn new_by_max(v: f64) -> Self {
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
        // event force_to_max force_to_min
    }

    /// 瞬时效果 返回对当前值的修改值 如造成伤害时处理护盾血量逻辑
    ///
    /// 无需手动修正属性值
    pub fn use_inst_effect<E>(&mut self, e: DynPropInstEffect<S>, prop_events: &mut E) -> f64
    where
        E: DynPropEvents<S>,
    {
        let real_eff = e.convert_real_effect(&self);
        self.alter_current_value(&real_eff, prop_events)
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
        let (target, attr_eff) = e.convert_attr_effect();
        match target {
            DynPropDurEffectTarget::ForMax => self.put_max_attr_effect(attr_eff),
            DynPropDurEffectTarget::ForMin => self.put_min_attr_effect(attr_eff),
        }
    }

    /// 移除持久效果（上下界限同时移除） 可外部调用
    ///
    /// 而后需 **手动调用** 刷新属性值 [`Self::refresh_value`]
    pub fn del_dur_effect(&mut self, s: &S) {
        self.the_max.del_effect(s);
        self.the_min.del_effect(s);
    }

    /// 获取当前所有的针对 max 的持久效果名称
    pub fn get_max_dur_effect_names(&self) -> Vec<S> {
        self.the_max.get_effect_names()
    }

    /// 根据名称获取当前针对 max 的持久效果名称
    pub fn get_max_dur_effect_by_name(&self, s: &S) -> Option<DynAttrEffect<S>> {
        self.the_max.get_effect_by_name(s)
    }

    /// 获取当前所有的针对 min 的持久效果名称
    pub fn get_min_dur_effect_names(&self) -> Vec<S> {
        self.the_min.get_effect_names()
    }

    /// 根据名称获取当前针对 min 的持久效果名称
    pub fn get_min_dur_effect_by_name(&self, s: &S) -> Option<DynAttrEffect<S>> {
        self.the_min.get_effect_by_name(s)
    }

    /// 给予一个持久效果的同时自动修改当前值 可外部调用
    ///
    /// 注意【仅增益效果会修改当前值（如提升最大生命值）】
    ///
    /// 无需手动调用刷新属性值
    pub fn do_put_dur_effect<E>(&mut self, e: DynPropDurEffect<S>, prop_events: &mut E)
    where
        E: DynPropEvents<S>,
    {
        self.put_dur_effect(e.clone());
        self.refresh_value();
        if let Some(real_eff) = e.convert_real_effect_for_max_buff(self) {
            self.alter_current_value(&real_eff, prop_events);
        }
    }

    /// 刷新周期效果的优先级列表
    pub fn refresh_period_effect(&mut self) {
        self.period_effects.refresh_order_keys();
    }

    /// 周期效果 可外部调用
    ///
    /// 而后需 **手动调用** 刷新属性值 [`Self::refresh_period_effect`]
    pub fn put_period_effect(&mut self, e: DynPropPeriodEffect<S>) {
        self.period_effects.put_or_stack_effect(e);
    }

    /// 移除周期效果 可外部调用
    ///
    /// 而后需 **手动调用** 刷新属性值 [`Self::refresh_period_effect`]
    pub fn del_period_effect(&mut self, s: &S) {
        self.period_effects.del_effect(s);
    }

    /// 重启周期效果 仅刷新时间和来源 不影响值
    ///
    /// 无需手动修正属性值
    pub fn restart_period_effect<T: ProxyEffect<S>>(&mut self, e: &T) {
        if let Some(eff) = self.period_effects.get_effect_mut(e.get_effect_name()) {
            eff.refresh_with_name(e);
        }
    }

    /// 获取当前所有的周期效果名称
    pub fn get_period_effect_names(&self) -> Vec<S> {
        self.period_effects.keys()
    }

    /// 根据名称获取当前周期效果
    pub fn get_period_effect_by_name(&self, s: &S) -> Option<DynPropPeriodEffect<S>> {
        self.period_effects.get_effect(s).cloned()
    }

    // todo 支持触发型效果
    // /// 当占比小于【一定比率】时 自动修改当前值 （用于斩杀或救命）
    // todo 是否应该将多个prop聚合成一个 如血量和护盾的关系
    // 优点 能在框架内进行测试验证
    // 优点 做触发效果时较为内聚（思考如何实现，是否基于游戏引擎去解耦开，传入或返回一个闭包）
    // 缺点 不同类型的伤害护盾计算逻辑可能需要在框架写死（致命）

    /// 无需手动刷新属性值
    pub fn process_time<E>(&mut self, delta: f64, prop_events: &mut E)
    where
        E: DynPropEvents<S>,
    {
        self.the_max.process_time(delta);
        self.the_min.process_time(delta);
        self.fix_current();

        let mut period_changed = false;
        for ele in self.period_effects.keys() {
            let Some(eff) = self.period_effects.get_effect_mut(&ele) else {
                continue;
            };

            let periods = eff.process_period(delta);
            let eff = eff.clone(); // 中断self的借用 之后不应该再对该类型做更改

            if eff.is_expired() {
                self.period_effects.del_effect(&ele);
                period_changed = true;
            } else if periods > 0 {
                let inst_eff = eff.convert_prop_inst_effect(&self);
                let real_eff = inst_eff.convert_real_effect(&self);
                for _ in 0..periods {
                    self.alter_current_value(&real_eff, prop_events);
                }
            }
        }

        if period_changed {
            self.refresh_period_effect();
        }
    }

    // =================================================================================

    /// 赋予最大值效果 仅effect内部调用 需要刷新
    pub(crate) fn put_max_attr_effect(&mut self, e: DynAttrEffect<S>) {
        self.the_max.put_or_stack_effect(e);
    }

    /// 赋予最小值效果 仅effect内部调用 需要刷新
    pub(crate) fn put_min_attr_effect(&mut self, e: DynAttrEffect<S>) {
        self.the_min.put_or_stack_effect(e);
    }

    /// 如对血量直接造成伤害 return delta 用于处理护盾逻辑 无需再次刷新
    fn alter_current_value<E>(&mut self, e: &Effect<S>, prop_events: &mut E) -> f64
    where
        E: DynPropEvents<S>,
    {
        let the_old = self.get_current();
        self.current += e.get_value();
        self.fix_current();
        let the_new = self.get_current();
        let the_delta = the_new - the_old;

        if the_delta < 0.0 && the_new == self.get_min() {
            prop_events.once_current_to_min(e);
        }
        the_delta
    }
}

// =================================================================================

#[cfg(test)]
mod tests {

    use crate::{
        attrs::event_prop::{DynPropEventsImpl, create_empty_events},
        effects::{duration_effect::EffectBuilder, native_effect::Effect},
    };

    use super::*;

    #[test]
    fn event_once_cur_min() {
        let mut prop: DynProp = DynProp::new_by_max(10.0);

        let mut count = "".to_string();

        for i in 0..5 {
            prop.alter_current_value(
                &Effect::new(format!("killed_by_{}", i), "effect_name".to_string(), -4.6),
                &mut DynPropEventsImpl {
                    once_current_to_min: Some(|eff: &Effect<String>| {
                        count = eff.get_from_name().to_string();
                    }),
                    ..Default::default()
                },
            );
        }

        assert_eq!(count, "killed_by_2");
    }

    #[test]
    fn put_dur_effect_each_max_min() {
        let mut prop: DynProp = DynProp::new_by_max(100.0);
        prop.put_dur_effect(DynPropDurEffect::new_max_per(EffectBuilder::new_infinite(
            "from_name",
            "effect_name",
            -0.5,
        )));
        prop.refresh_value();
        assert_eq!(prop.get_max(), 50.0);
        assert_eq!(prop.get_current(), 50.0);

        let mut prop: DynProp = DynProp::new_by_max(100.0);
        prop.put_dur_effect(DynPropDurEffect::new_max_per(EffectBuilder::new_infinite(
            "from_name",
            "effect_name",
            0.5,
        )));
        prop.refresh_value();
        assert_eq!(prop.get_max(), 150.0);
        assert_eq!(prop.get_current(), 100.0);

        let mut prop: DynProp = DynProp::new_by_max(100.0);
        prop.put_dur_effect(DynPropDurEffect::new_max_val(EffectBuilder::new_infinite(
            "from_name",
            "effect_name",
            -50.0,
        )));
        prop.refresh_value();
        assert_eq!(prop.get_max(), 50.0);
        assert_eq!(prop.get_current(), 50.0);

        let mut prop: DynProp = DynProp::new_by_max(100.0);
        prop.put_dur_effect(DynPropDurEffect::new_max_val(EffectBuilder::new_infinite(
            "from_name",
            "effect_name",
            100.0,
        )));
        prop.put_dur_effect(DynPropDurEffect::new_min_val(EffectBuilder::new_infinite(
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
        let mut prop: DynProp = DynProp::new_by_max(100.0);
        assert_eq!(prop.get_current(), 100.0);

        prop.use_inst_effect(
            DynPropInstEffect::new_val(EffectBuilder::new_instant("someone", "effect_name", -50.0)),
            &mut create_empty_events(),
        );
        assert_eq!(prop.get_current(), 50.0);

        prop.use_inst_effect(
            DynPropInstEffect::new_val(EffectBuilder::new_instant("someone", "effect_name", -50.0)),
            &mut create_empty_events(),
        );
        assert_eq!(prop.get_current(), 0.0);

        prop.use_inst_effect(
            DynPropInstEffect::new_val(EffectBuilder::new_instant("someone", "effect_name", -50.0)),
            &mut create_empty_events(),
        );
        assert_eq!(prop.get_current(), 0.0);

        prop.use_inst_effect(
            DynPropInstEffect::new_val(EffectBuilder::new_instant("someone", "effect_name", 200.0)),
            &mut create_empty_events(),
        );
        assert_eq!(prop.get_current(), 100.0);
    }

    #[test]
    fn do_put_dur_effect() {
        let mut prop: DynProp = DynProp::new_by_max(50.0);
        let eff = DynPropDurEffect::new_max_per(EffectBuilder::new_infinite(
            "from_name",
            "effect_name1",
            0.2,
        ));
        prop.do_put_dur_effect(eff, &mut create_empty_events());
        assert_eq!(prop.get_current(), 60.0);

        prop.current = 20.0;
        let eff = DynPropDurEffect::new_max_val(EffectBuilder::new_infinite(
            "from_name",
            "effect_name2",
            10.0,
        ));
        prop.do_put_dur_effect(eff, &mut create_empty_events());
        assert_eq!(prop.get_current(), 30.0);
        assert_eq!(prop.get_max(), 70.0);
    }

    #[test]
    fn put_period_effect_each_cur() {
        let mut prop: DynProp = DynProp::new_by_max(100.0);
        let eff = DynPropPeriodEffect::new_val(
            EffectBuilder::new_infinite("from_name", "effect_name", -10.0),
            1.0,
        );
        prop.put_period_effect(eff);
        prop.refresh_period_effect();
        assert_eq!(prop.get_period_effect_names().iter().count(), 1);
        assert_eq!(prop.get_current(), 100.0);
        prop.process_time(0.5, &mut create_empty_events());
        assert_eq!(prop.get_current(), 100.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 90.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 80.0);

        let mut prop: DynProp = DynProp::new_by_max(100.0);
        let eff = DynPropPeriodEffect::new_cur_per(
            EffectBuilder::new_infinite("from_name", "effect_name", -0.1),
            1.0,
        );
        prop.put_period_effect(eff);
        prop.refresh_period_effect();
        assert_eq!(prop.get_current(), 100.0);
        prop.process_time(0.5, &mut create_empty_events());
        assert_eq!(prop.get_current(), 100.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 90.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 81.0);

        let mut prop: DynProp = DynProp::new_by_max(100.0);
        let eff = DynPropPeriodEffect::new_max_per(
            EffectBuilder::new_infinite("from_name", "effect_name", -0.1),
            1.0,
        );
        prop.put_period_effect(eff);
        prop.refresh_period_effect();
        assert_eq!(prop.get_current(), 100.0);
        prop.process_time(0.5, &mut create_empty_events());
        assert_eq!(prop.get_current(), 100.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 90.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 80.0);
    }

    #[test]
    fn put_period_effect_to_val_near() {
        let mut prop: DynProp = DynProp::new_by_max(100.0);
        let mut eff = DynPropPeriodEffect::new_cur_val_to_val(
            EffectBuilder::new_infinite("from_name", "effect_name", 9.0),
            50.0,
            1.0,
        );
        eff.set_max_stack(2);
        eff.set_stack(2);
        prop.put_period_effect(eff);
        prop.refresh_period_effect();

        // 持续逼近
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 100.0 - 18.0 * 1.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 100.0 - 18.0 * 2.0);
        // 达到目标值
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 50.0);

        // 另一个方向
        prop.current = 0.0;

        // 持续逼近
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 18.0 * 1.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 18.0 * 2.0);
        // 达到目标值
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 50.0);
    }

    #[test]
    fn put_period_effect_to_val_far() {
        let mut prop: DynProp = DynProp::new_by_max(100.0);
        prop.current = 50.0;
        let eff = DynPropPeriodEffect::new_cur_val_to_val(
            EffectBuilder::new_infinite("from_name", "effect_name", -9.0),
            50.0,
            1.0,
        );
        prop.put_period_effect(eff);
        prop.refresh_period_effect();

        // 持续逼近
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 50.0 - 9.0 * 1.0);
        prop.process_time(2.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 50.0 - 9.0 * 3.0);
        // 达到目标值
        prop.process_time(3.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 0.0);

        // 另一个方向
        prop.current = 51.0;

        // 持续逼近
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 51.0 + 9.0 * 1.0);
        prop.process_time(2.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 51.0 + 9.0 * 3.0);
        // 达到目标值
        prop.process_time(3.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 100.0);
    }

    #[test]
    fn put_period_effect_to_val_twice() {
        let mut prop: DynProp = DynProp::new_by_max(100.0);
        let eff = DynPropPeriodEffect::new_cur_val_to_val(
            EffectBuilder::new_infinite("from_name", "1", 20.0),
            50.0,
            6.0,
        );
        prop.put_period_effect(eff);
        let eff = DynPropPeriodEffect::new_cur_val_to_val(
            EffectBuilder::new_infinite("from_name", "2", 3.0),
            10.0,
            1.0,
        );
        prop.put_period_effect(eff);
        prop.refresh_period_effect();

        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 100.0 - 3.0 * 1.0 - 20.0 * 0.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 100.0 - 3.0 * 2.0 - 20.0 * 0.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 100.0 - 3.0 * 3.0 - 20.0 * 0.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 100.0 - 3.0 * 4.0 - 20.0 * 0.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 100.0 - 3.0 * 5.0 - 20.0 * 0.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 100.0 - 3.0 * 6.0 - 20.0 * 1.0); // 6.0s
        assert_eq!(prop.get_current(), 62.0);

        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 100.0 - 3.0 * 7.0 - 20.0 * 1.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 100.0 - 3.0 * 8.0 - 20.0 * 1.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 100.0 - 3.0 * 9.0 - 20.0 * 1.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 100.0 - 3.0 * 10.0 - 20.0 * 1.0);
        assert_eq!(prop.get_current(), 50.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 100.0 - 3.0 * 11.0 - 20.0 * 1.0);
        assert_eq!(prop.get_current(), 47.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 50.0 - 3.0); // 6.0s 小于 50 因此 +20.0 且先执行

        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 50.0 - 3.0 * 2.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 50.0 - 3.0 * 3.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 50.0 - 3.0 * 4.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 50.0 - 3.0 * 5.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 50.0 - 3.0 * 6.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 50.0 - 3.0 * 1.0); // 6.0s
    }

    #[test]
    fn restart_for_wait_and_expired_time() {
        let mut prop: DynProp = DynProp::new_by_max(100.0);
        let mut eff = DynPropPeriodEffect::new_val(
            EffectBuilder::new_duration("from_name", "1", -10.0, 10.0),
            1.0,
        );
        eff.set_wait_time(5.0);
        prop.put_period_effect(eff);
        prop.refresh_period_effect();

        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 100.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 100.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 100.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 100.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 100.0); // 5.0s wait 结束
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 90.0); // 6.0s 第一个周期
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 80.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 70.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 60.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 60.0); // 10.0s 恰好没触发
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 60.0);

        // new one
        let mut eff = DynPropPeriodEffect::new_val(
            EffectBuilder::new_duration("from_name", "1", -10.0, 5.0),
            1.0,
        );
        eff.set_wait_time(2.0);
        prop.put_period_effect(eff);
        prop.refresh_period_effect();

        // restart_for_wait_time
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 60.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 60.0);
        prop.restart_period_effect(&Effect::new("from_name", "1", 1.0));
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 60.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 60.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 50.0); // 3.0s 第一个周期

        // restart_for_expired_time
        prop.restart_period_effect(&Effect::new("from_name", "1", 1.0));
        prop.process_time(2.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 50.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 40.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 30.0);
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 30.0); // 5.0s 老化
    }

    #[test]
    fn put_for_stack() {
        let mut prop: DynProp = DynProp::new_by_max(100.0);
        let mut eff =
            DynPropPeriodEffect::new_val(EffectBuilder::new_infinite("from_name", "1", -1.0), 1.0);
        eff.set_max_stack(3);
        prop.put_period_effect(eff);
        prop.refresh_period_effect();

        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 99.0); // -1

        prop.put_period_effect(DynPropPeriodEffect::new_val(
            EffectBuilder::new_infinite("from_name", "1", -1.0),
            1.0,
        ));
        prop.refresh_period_effect();
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 97.0); // -2

        prop.put_period_effect(DynPropPeriodEffect::new_val(
            EffectBuilder::new_infinite("from_name", "1", -1.0),
            1.0,
        ));
        prop.refresh_period_effect();
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 94.0); // -3

        prop.put_period_effect(DynPropPeriodEffect::new_val(
            EffectBuilder::new_infinite("from_name", "1", -1.0),
            1.0,
        ));
        prop.refresh_period_effect();
        prop.process_time(1.0, &mut create_empty_events());
        assert_eq!(prop.get_current(), 91.0); // -3 max
    }
}

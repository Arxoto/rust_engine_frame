use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr_prop::{
        attr_eff::AttrModifier,
        attrs::Attr,
        prop_bounds_eff::{PropBoundsEffect, PropBoundsEffectTarget},
    },
};

#[derive(Debug)]
pub struct PropAlterResult {
    /// 实际生效值
    pub real_eff_val: f64,
}

/// property 属性 一般用作角色资源槽 可被效果影响
#[derive(Debug, Default)]
pub struct Prop {
    upper: Attr,
    lower: Attr,
    current: f64,
}

impl Prop {
    pub fn new(current: f64, upper: f64, lower: f64) -> Self {
        Self {
            upper: Attr::new(upper),
            lower: Attr::new(lower),
            current,
        }
    }

    pub fn get_max_origin(&self) -> f64 {
        self.upper.get_origin()
    }

    pub fn get_max(&self) -> f64 {
        self.upper.get_current()
    }

    pub fn get_current(&self) -> f64 {
        self.current
    }

    /// 如：判断当前血量是否为零，然后记录致命来源
    pub fn current_is_zero(&self) -> bool {
        self.current == 0.0
    }

    /// 应用上下限（公开钳制入口）
    ///
    /// 配合 [`Prop::refresh_bounds`] 使用：刷新上下限后调用本方法，
    /// 将当前值重新钳制到新的上下限范围内。
    pub fn apply_bounds(&mut self) {
        let mut value = self.current;
        value = self.upper.get_current().min(value);
        value = self.lower.get_current().max(value);
        self.current = value;
    }

    /// 刷新上下限数值
    ///
    /// 【注意】不会自动应用上下限，只会需手动调用
    pub fn refresh_bounds<'a, S: FixedName + 'a, Timer: 'a>(
        &mut self,
        effs: impl Iterator<Item = &'a PropBoundsEffect<S, Timer>>,
    ) {
        let mut upper_modifier = AttrModifier::default();
        let mut lower_modifier = AttrModifier::default();

        for ele in effs {
            match ele.get_target() {
                PropBoundsEffectTarget::Upper => upper_modifier.reduce(ele.get_eff()),
                PropBoundsEffectTarget::Lower => lower_modifier.reduce(ele.get_eff()),
            }
        }

        self.upper.apply_modify(&upper_modifier);
        self.lower.apply_modify(&lower_modifier);
    }

    /// 考虑到伤害公式的计算，这里只支持绝对值
    ///
    /// 建议先把所有伤害 eff 先聚合后，再一次进行计算
    ///
    /// - 有助于：减少计算次数，并且防止血量本身较多时的单次治疗超上限
    /// - 不适用：血量护盾这种不同伤害类型影响不同层级的复杂逻辑
    ///
    /// 应先复制当前值和上下限作为基准，然后计算百分比得出绝对值、推入伤害队列，之后再 [`Prop::refresh_bounds`] [`Prop::apply_bounds`]
    pub fn apply_eff(&mut self, val: f64) -> PropAlterResult {
        let old_value = self.current;

        self.current += val;
        self.apply_bounds();

        let real_eff_val = self.current - old_value;
        PropAlterResult { real_eff_val }
    }

    /// 只有当前值足够才会去应用效果（如法力不够则施放失败）
    ///
    /// 考虑特殊场景，在一帧内同时发生如下事件
    /// - 一个治疗效果、一个伤害效果（一帧结束后最终要保证治疗效果不丢失）
    /// - 一个自动触发效果，自动扣一定量的血量获得某种效果，但会保证扣除后的血量大于某个值（判断当前值必须是即时的，否则会导致误伤）
    /// - 根据当前血量的值获得某种效果（在任何时候都必须在上下限范围内，与计算中间值最后钳制的机制冲突）
    ///
    /// 为了保证始终准确，设计整个属性效果管线
    /// - 获取当前值作为一个快照 Snapshot ，后续一帧内的所有读取【真实值】的操作都读取这个视图
    /// - 无论如何都生效的效果走 buffer 缓冲，防止治疗效果丢失
    /// - “时序耦合”的效果最后依次计算，读取【计算中间实时值】，并且编译期就要确定唯一顺序
    /// - 一帧的末尾钳制刷新【真实值】，作为事实依据
    /// todo 实现这个属性效果管线，将 prop 的 max 和 min 属性作为可选钳制 Clamp
    pub fn apply_eff_checked(&mut self, val: f64, want_ge: f64) -> Option<PropAlterResult> {
        if self.current + val >= want_ge {
            Some(self.apply_eff(val))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base_lib::{
        cores::timers::tick_timer::TickTimer,
        eff_attr_prop::{effects::Effect, prop_bounds_eff::PropBoundsEffectType},
    };

    /// 等级 A 演示：文档承诺的"刷新上下限 → 应用钳制"公开序列可执行
    ///
    /// 提升上限 → 当前值随之抬升；效果过期移除后，`apply_bounds` 将当前值重新钳制回边界。
    #[test]
    fn refresh_bounds_then_apply_bounds_clamps_current() {
        let mut prop = Prop::new(100.0, 100.0, 0.0);

        // 效果提高上限至 150，当前值随之抬升到 140
        let eff = PropBoundsEffect::new(
            PropBoundsEffectType::UpperAdd,
            Effect::new("buff", "max_hp", 50.0),
            TickTimer::inf(),
        );
        prop.refresh_bounds(std::iter::once(&eff));
        prop.apply_eff(40.0);
        assert_eq!(prop.get_max(), 150.0);
        assert_eq!(prop.get_current(), 140.0);

        // 效果过期移除 → 上限回落，apply_bounds 将当前值重新钳制到边界
        let effs: Vec<PropBoundsEffect<&str, TickTimer>> = Vec::new();
        prop.refresh_bounds(effs.iter());
        assert_eq!(prop.get_max(), 100.0);
        assert_eq!(prop.get_current(), 140.0); // 未自动钳制
        prop.apply_bounds();
        assert_eq!(prop.get_current(), 100.0); // 回落到边界
    }

    /// 等级 A 演示：低于下限时被钳制
    #[test]
    fn apply_bounds_clamps_to_lower() {
        let mut prop = Prop::new(-10.0, 100.0, 0.0);
        prop.apply_bounds();
        assert_eq!(prop.get_current(), 0.0);
    }

    /// 瞬时变更：apply_eff 修改当前值并返回实际生效值
    #[test]
    fn prop_apply_eff_changes_current() {
        let mut prop = Prop::new(100.0, 100.0, 0.0);
        let res = prop.apply_eff(-30.0);
        assert_eq!(prop.get_current(), 70.0);
        assert_eq!(res.real_eff_val, -30.0);
    }

    /// 瞬时变更被上下限钳制，real_eff_val 反映实际生效量
    #[test]
    fn prop_apply_eff_clamps_to_bounds() {
        let mut prop = Prop::new(100.0, 100.0, 0.0);
        let res = prop.apply_eff(-200.0);
        assert_eq!(prop.get_current(), 0.0); // 钳制到下限
        assert_eq!(res.real_eff_val, -100.0); // 实际只生效了 100

        let mut prop = Prop::new(100.0, 100.0, 0.0);
        let res = prop.apply_eff(50.0);
        assert_eq!(prop.get_current(), 100.0); // 钳制到上限
        assert_eq!(res.real_eff_val, 0.0);
    }

    /// apply_eff_checked：当前值足够才生效，否则返回 None 且不改变
    #[test]
    fn prop_apply_eff_checked_gates_on_current() {
        let mut prop = Prop::new(30.0, 100.0, 0.0);
        // 30 - 20 = 10 >= 0 → 生效
        assert!(prop.apply_eff_checked(-20.0, 0.0).is_some());
        assert_eq!(prop.get_current(), 10.0);

        // 10 - 20 = -10 < 0 → 不生效
        assert!(prop.apply_eff_checked(-20.0, 0.0).is_none());
        assert_eq!(prop.get_current(), 10.0);
    }

    /// 当前值为 0 时判定归零（用于记录致命来源）
    #[test]
    fn prop_current_is_zero() {
        let mut prop = Prop::new(100.0, 100.0, 0.0);
        assert!(!prop.current_is_zero());
        prop.apply_eff(-100.0);
        assert!(prop.current_is_zero());
    }

    /// refresh_bounds 重算下限，apply_bounds 将当前值钳回新下限
    #[test]
    fn prop_refresh_bounds_recomputes_lower() {
        let mut prop = Prop::new(100.0, 100.0, 0.0);
        let eff = PropBoundsEffect::new(
            PropBoundsEffectType::LowerAdd,
            Effect::new("buff", "min_hp", 20.0),
            TickTimer::inf(),
        );
        prop.refresh_bounds(std::iter::once(&eff));

        prop.apply_eff(-50.0);
        assert_eq!(prop.get_current(), 50.0); // 高于新下限 20，不钳制
        prop.apply_eff(-40.0);
        assert_eq!(prop.get_current(), 20.0); // 钳制到新下限
    }
}

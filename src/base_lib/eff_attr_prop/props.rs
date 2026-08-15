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

    /// 应用上下限
    fn apply_bounds(&mut self) {
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
    pub fn apply_eff(&mut self, eff_val: f64) -> PropAlterResult {
        let old_value = self.current;

        self.current += eff_val;
        self.apply_bounds();

        let real_eff_val = self.current - old_value;
        PropAlterResult { real_eff_val }
    }

    /// 只有当前值足够才会去应用效果（如法力不够则施放失败）
    pub fn apply_eff_checked(&mut self, eff_val: f64, ge: f64) -> Option<PropAlterResult> {
        if self.current + eff_val >= ge {
            Some(self.apply_eff(eff_val))
        } else {
            None
        }
    }
}

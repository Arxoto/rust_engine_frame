use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr_prop::{
        attr_eff::AttrModifier,
        attrs::Attr,
        effects::EffectMeaning,
        prop_bounds_eff::{PropBoundsEffect, PropBoundsEffectTarget},
        prop_eff::PropEffect,
    },
};

#[derive(Debug, Default)]
pub struct PropAlterResult<S: FixedName> {
    /// 本次应用效果中的第一个有害来源
    pub first_bad: Option<S>,
    /// 当前值是否小于等于零（若最小值大于零则等于 false ）
    pub current_le_zero: bool,
}

impl<S: FixedName> PropAlterResult<S> {
    /// 因什么导致的清零（返回造成有害效果的来源）
    pub fn le_zero_by(&self) -> Option<&S> {
        if self.current_le_zero {
            self.first_bad.as_ref()
        } else {
            None
        }
    }
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

    fn apply_bounds(&mut self) {
        let mut value = self.current;
        value = self.upper.get_current().min(value);
        value = self.lower.get_current().max(value);
        self.current = value;
    }

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

    /// 风格是把所有修改存入 buffer 然后一把梭哈
    ///
    /// 应先复制当前值和上下限作为基准，然后计算百分比得出绝对值、推入buffer，之后再 [`Prop::refresh_bounds`] [`Prop::apply_bounds`]
    pub fn apply_effs<'a, S: FixedName + 'a, Timer: 'a>(
        &mut self,
        buffer: impl Iterator<Item = &'a PropEffect<S>>,
    ) -> PropAlterResult<S> {
        let mut first_bad: Option<S> = None;

        for ele in buffer {
            if first_bad.is_none() && ele.which_nature().is_bad() {
                first_bad = Some(ele.get_from_name().clone());
            }
            self.current += ele.get_effect_value();
        }

        self.apply_bounds();

        PropAlterResult {
            first_bad,
            current_le_zero: self.current <= 0.0,
        }
    }
}

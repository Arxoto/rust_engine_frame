use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr_prop::attr_eff::{AttrEffect, AttrModifier},
};

/// attribute 属性 一般用作角色属性值 可被效果影响
///
/// 另一种风格是将所有基础属性展开平铺，每个效果直接修改对应属性，灵活但复杂
#[derive(Debug, Default)]
pub struct Attr {
    origin: f64,
    current: f64,
}

impl Attr {
    pub fn new(origin: f64) -> Self {
        Self {
            origin,
            current: origin,
        }
    }

    pub fn get_origin(&self) -> f64 {
        self.origin
    }

    pub fn get_current(&self) -> f64 {
        self.current
    }

    /// 刷新属性，在效果更新后
    pub fn refresh_value<'a, S: FixedName + 'a, Timer: 'a>(
        &mut self,
        effs: impl Iterator<Item = &'a AttrEffect<S, Timer>>,
    ) {
        let mut attr_modifier = AttrModifier::default();

        for ele in effs {
            attr_modifier.reduce(ele);
        }

        self.apply_modify(&attr_modifier);
    }

    pub(super) fn apply_modify(&mut self, am: &AttrModifier) {
        self.current = am.apply_modify(self.origin)
    }
}

// todo test

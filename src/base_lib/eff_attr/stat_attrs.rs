use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr::stat_attr_effs::{StatAttrEff, StatAttrModifier},
};

/// 状态属性，常用于各种系统的源端，比如 “攻击力/防御力”
///
/// 另一种风格是将所有基础属性展开平铺，每个效果直接修改对应属性，灵活但复杂
#[derive(Debug)]
pub struct StatAttr {
    /// 原始值，未经过修改器修改
    origin: f64,
    /// 当前值，经过修改器修改
    current: f64,
}

impl StatAttr {
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
        effs: impl Iterator<Item = &'a StatAttrEff<S, Timer>>,
    ) {
        let mut modifier = StatAttrModifier::default();

        for ele in effs {
            modifier.reduce(ele);
        }

        self.current = modifier.apply_modify(self.origin)
    }
}

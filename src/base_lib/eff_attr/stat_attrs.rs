use crate::base_lib::eff_attr::{
    modifier_collections::ModifiableAttr,
    stat_attr_modifiers::{StatAttrAggregator, StatAttrModifier},
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
}

impl ModifiableAttr<StatAttrModifier> for StatAttr {
    fn get_origin(&self) -> f64 {
        self.origin
    }

    fn get_current(&self) -> f64 {
        self.current
    }

    fn refresh_value<'a>(&mut self, modifiers: impl Iterator<Item = &'a StatAttrModifier>)
    where
        StatAttrModifier: 'a,
    {
        let mut aggregator = StatAttrAggregator::default();

        for ele in modifiers {
            aggregator.reduce(ele);
        }

        self.current = aggregator.apply_modify(self.origin)
    }
}

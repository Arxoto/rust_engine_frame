use crate::base_lib::eff_attr::{
    bound_attr_modifiers::{BoundAttrAggregator, BoundAttrModifier},
    modifier_collections::ModifiableAttr,
};

/// 边界约束属性，用于限制某属性的上下界限
///
/// 对应效果的计算只能基于基础值，因此修改值可预测，可用作各种公式计算的锚点
/// - 如：“增加血量上限的同时增加等量的血量”
#[derive(Debug)]
pub struct BoundAttr {
    /// 原始值，未经过修改器修改
    origin: f64,
    /// 当前值，经过修改器修改
    current: f64,
}

impl BoundAttr {
    pub fn new(origin: f64) -> Self {
        Self {
            origin,
            current: origin,
        }
    }
}

impl ModifiableAttr<BoundAttrModifier> for BoundAttr {
    fn get_origin(&self) -> f64 {
        self.origin
    }

    fn get_current(&self) -> f64 {
        self.current
    }

    fn refresh_value<'a>(&mut self, modifiers: impl Iterator<Item = &'a BoundAttrModifier>)
    where
        BoundAttrModifier: 'a,
    {
        let mut aggregator = BoundAttrAggregator::default();

        for ele in modifiers {
            aggregator.reduce(ele);
        }

        self.current = aggregator.apply_modify(self.origin);
    }
}

/// 有界属性的约束值，用于自动转换
#[derive(Debug)]
pub struct BoundValue(f64);

impl BoundValue {
    #[inline]
    pub fn get_value(v: impl Into<BoundValue>) -> f64 {
        let value: Self = v.into();
        value.0
    }
}

impl From<f64> for BoundValue {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl From<&BoundAttr> for BoundValue {
    fn from(value: &BoundAttr) -> Self {
        Self(value.get_current())
    }
}

/// 上下界限约束
#[derive(Debug)]
pub struct BoundRange<Lower, Upper>
where
    BoundValue: From<Lower>,
    BoundValue: From<Upper>,
{
    pub lower: Lower,
    pub upper: Upper,
}

impl<Lower, Upper> BoundRange<Lower, Upper>
where
    BoundValue: From<Lower>,
    BoundValue: From<Upper>,
{
    #[inline]
    pub fn clamp(self, v: f64) -> f64 {
        let lower = BoundValue::get_value(self.lower);
        let upper = BoundValue::get_value(self.upper);
        lower.max(upper.min(v))
    }
}

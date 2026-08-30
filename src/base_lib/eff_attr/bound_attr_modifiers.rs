use crate::base_lib::eff_attr::{
    aggregators::{ADDITION_BASE_LINE, BasicAggregator, InvalidModifier, PERCENT_BASE_LINE},
    effects::{EffectMean, EffectMeaning},
};

/// 属性修改维度
///
/// 计算公式见 [`BoundAttrAggregator::apply_modify`]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BoundAttrModifyDimension {
    /// 基础加法（描述参考：基础力量增加xx），可安全使用
    BasicAdd,
    /// 基础百分比（描述参考：基础力量提升xx%），可安全使用
    BasicPer,
}

impl BoundAttrModifyDimension {
    pub fn calc_real_val(&self, base: f64, val: f64) -> f64 {
        match self {
            BoundAttrModifyDimension::BasicAdd => val,
            BoundAttrModifyDimension::BasicPer => base * val,
        }
    }
}

/// 属性修改器
///
/// 若想在修改上限的同时修改实际值，那么需要同时生成【修改上限】的效果和【修改实际值】的效果
///
/// 为了保证两者修改效果一致，限制修改维度只能基于基础值修改（不会被放大缩小产生偏差）
#[derive(Clone, Debug)]
pub struct BoundAttrModifier {
    /// 修改维度 对应公式变量
    dimension: BoundAttrModifyDimension,
    /// 修改效果
    value: f64,
}

impl BoundAttrModifier {
    pub fn new(dimension: BoundAttrModifyDimension, value: f64) -> Self {
        Self { dimension, value }
    }

    pub fn get_dimension(&self) -> BoundAttrModifyDimension {
        self.dimension
    }
}

impl EffectMeaning for BoundAttrModifier {
    fn which_nature(&self) -> EffectMean {
        let value = self.value;
        match self.dimension {
            BoundAttrModifyDimension::BasicAdd => {
                EffectMean::which_nature(value, ADDITION_BASE_LINE)
            }
            BoundAttrModifyDimension::BasicPer => {
                EffectMean::which_nature(value, PERCENT_BASE_LINE)
            }
        }
    }
}

impl InvalidModifier for BoundAttrModifier {
    fn new_invalid() -> Self {
        Self {
            dimension: BoundAttrModifyDimension::BasicAdd,
            value: ADDITION_BASE_LINE,
        }
    }
}

/// 属性效果修改器
///
/// 计算公式见 [`BoundAttrAggregator::apply_modify`]
#[derive(Debug, Default)]
pub struct BoundAttrAggregator(BasicAggregator);

impl BoundAttrAggregator {
    pub fn reduce(&mut self, modifier: &BoundAttrModifier) {
        let v = modifier.value;

        match modifier.dimension {
            BoundAttrModifyDimension::BasicAdd => self.0.reduce_add(v),
            BoundAttrModifyDimension::BasicPer => self.0.reduce_pct(v),
        }
    }

    pub fn apply_modify(&self, v: f64) -> f64 {
        self.0.apply_modify(v)
    }
}

// todo test

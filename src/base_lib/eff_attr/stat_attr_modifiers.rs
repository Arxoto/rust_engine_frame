use crate::base_lib::eff_attr::{
    aggregators::{
        ADDITION_BASE_LINE, AdvancedAggregator, InvalidModifier, MULT_BASE_LINE, PERCENT_BASE_LINE,
    },
    effects::{EffectMean, EffectMeaning},
};

/// 属性修改维度
///
/// 计算公式见 [`StatAttrAggregator::apply_modify`]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StatAttrModifyDimension {
    /// 基础加法（描述参考：基础力量增加xx），可安全使用
    BasicAdd,
    /// 基础百分比（描述参考：基础力量提升xx%），可安全使用
    BasicPer,
    /// 百分比（描述参考：力量提升xx%），可安全使用
    FinalPer,
    /// 乘法（描述参考：力量变为原先的xx倍），指数增长、谨慎使用
    FinalMul,
}

/// 属性修改器
#[derive(Clone, Debug)]
pub struct StatAttrModifier {
    /// 修改维度 对应公式变量
    dimension: StatAttrModifyDimension,
    /// 修改效果
    value: f64,
}

impl StatAttrModifier {
    pub fn new(dimension: StatAttrModifyDimension, value: f64) -> Self {
        Self { dimension, value }
    }

    pub fn get_type(&self) -> StatAttrModifyDimension {
        self.dimension
    }
}

impl EffectMeaning for StatAttrModifier {
    fn which_nature(&self) -> EffectMean {
        let value = self.value;
        match self.dimension {
            StatAttrModifyDimension::BasicAdd => {
                EffectMean::which_nature(value, ADDITION_BASE_LINE)
            }
            StatAttrModifyDimension::BasicPer => EffectMean::which_nature(value, PERCENT_BASE_LINE),
            StatAttrModifyDimension::FinalPer => EffectMean::which_nature(value, PERCENT_BASE_LINE),
            StatAttrModifyDimension::FinalMul => EffectMean::which_nature(value, MULT_BASE_LINE),
        }
    }
}

impl InvalidModifier for StatAttrModifier {
    fn new_invalid() -> Self {
        Self {
            dimension: StatAttrModifyDimension::BasicAdd,
            value: ADDITION_BASE_LINE,
        }
    }
}

/// 属性效果修改器
///
/// 计算公式见 [`StatAttrAggregator::apply_modify`]
#[derive(Debug, Default)]
pub struct StatAttrAggregator(AdvancedAggregator);

impl StatAttrAggregator {
    pub fn reduce(&mut self, modifier: &StatAttrModifier) {
        let v = modifier.value;

        match modifier.dimension {
            StatAttrModifyDimension::BasicAdd => self.0.reduce_basic_add(v),
            StatAttrModifyDimension::BasicPer => self.0.reduce_basic_pct(v),
            StatAttrModifyDimension::FinalPer => self.0.reduce_final_pct(v),
            StatAttrModifyDimension::FinalMul => self.0.reduce_final_mult(v),
        }
    }

    pub fn apply_modify(&self, v: f64) -> f64 {
        self.0.apply_modify(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造一个指定类型与数值的持久属性效果
    fn make_eff(eff_type: StatAttrModifyDimension, eff_value: f64) -> StatAttrModifier {
        StatAttrModifier::new(eff_type, eff_value)
    }

    /// 默认修改器是恒等变换
    #[test]
    fn test_default_modifier_is_identity() {
        let am = StatAttrAggregator::default();
        assert_eq!(am.apply_modify(100.0), 100.0);
        assert_eq!(am.apply_modify(0.0), 0.0);
    }

    /// 单一类型效果的累加
    #[test]
    fn test_reduce_single_type_accumulation() {
        let mut am = StatAttrAggregator::default();
        am.reduce(&make_eff(StatAttrModifyDimension::BasicAdd, 5.0));
        am.reduce(&make_eff(StatAttrModifyDimension::BasicAdd, 3.0));
        assert_eq!(am.apply_modify(10.0), 18.0); // (8 + 10*1) * 1 * 1

        let mut am = StatAttrAggregator::default();
        am.reduce(&make_eff(StatAttrModifyDimension::BasicPer, 0.1));
        am.reduce(&make_eff(StatAttrModifyDimension::BasicPer, 0.2));
        assert_eq!(am.apply_modify(100.0), 130.0); // (0 + 100*1.3) * 1 * 1

        let mut am = StatAttrAggregator::default();
        am.reduce(&make_eff(StatAttrModifyDimension::FinalPer, 0.5));
        assert_eq!(am.apply_modify(100.0), 150.0); // (0 + 100*1) * 1.5 * 1

        let mut am = StatAttrAggregator::default();
        am.reduce(&make_eff(StatAttrModifyDimension::FinalMul, 2.0));
        am.reduce(&make_eff(StatAttrModifyDimension::FinalMul, 3.0));
        assert_eq!(am.apply_modify(100.0), 600.0); // (0 + 100*1) * 1 * 6
    }

    /// 混合类型按公式组合
    #[test]
    fn test_reduce_mixed_formula() {
        let mut am = StatAttrAggregator::default();
        am.reduce(&make_eff(StatAttrModifyDimension::BasicAdd, 10.0));
        am.reduce(&make_eff(StatAttrModifyDimension::BasicPer, 0.5));
        am.reduce(&make_eff(StatAttrModifyDimension::FinalPer, 0.25));
        am.reduce(&make_eff(StatAttrModifyDimension::FinalMul, 2.0));
        // (10 + 100*1.5) * 1.25 * 2 = 400
        assert_eq!(am.apply_modify(100.0), 400.0);
    }

    /// 边界值：负基础百分比削减基础值
    #[test]
    fn test_basic_per_negative_reduces() {
        let mut am = StatAttrAggregator::default();
        am.reduce(&make_eff(StatAttrModifyDimension::BasicPer, -0.5));
        assert_eq!(am.apply_modify(100.0), 50.0);
    }

    /// 边界值：负基础加值使结果低于基础值
    #[test]
    fn test_basic_add_negative() {
        let mut am = StatAttrAggregator::default();
        am.reduce(&make_eff(StatAttrModifyDimension::BasicAdd, -30.0));
        assert_eq!(am.apply_modify(100.0), 70.0);
    }

    /// 边界值：最终乘法为 0 时结果恒为 0
    #[test]
    fn test_final_mul_zero() {
        let mut am = StatAttrAggregator::default();
        am.reduce(&make_eff(StatAttrModifyDimension::FinalMul, 0.0));
        assert_eq!(am.apply_modify(100.0), 0.0);
    }

    /// 边界值：最终乘法为负时符号翻转
    #[test]
    fn test_final_mul_negative_flips_sign() {
        let mut am = StatAttrAggregator::default();
        am.reduce(&make_eff(StatAttrModifyDimension::FinalMul, -2.0));
        assert_eq!(am.apply_modify(100.0), -200.0);
    }

    /// 边界值：基础值为 0 时结果只取决于基础加法
    #[test]
    fn test_zero_origin() {
        let mut am = StatAttrAggregator::default();
        am.reduce(&make_eff(StatAttrModifyDimension::BasicPer, 2.0));
        am.reduce(&make_eff(StatAttrModifyDimension::FinalMul, 3.0));
        assert_eq!(am.apply_modify(0.0), 0.0);
    }

    /// 增益/减益判断：基础加法以 0 为基线
    #[test]
    fn test_meaning_basic_add() {
        assert!(
            make_eff(StatAttrModifyDimension::BasicAdd, 5.0)
                .which_nature()
                .is_good()
        );
        assert!(
            make_eff(StatAttrModifyDimension::BasicAdd, -5.0)
                .which_nature()
                .is_bad()
        );
        assert!(
            make_eff(StatAttrModifyDimension::BasicAdd, 0.0)
                .which_nature()
                .is_neutral()
        );
    }

    /// 增益/减益判断：基础百分比以 0 为基线（正增量为增益）
    #[test]
    fn test_meaning_basic_per() {
        assert!(
            make_eff(StatAttrModifyDimension::BasicPer, 0.5)
                .which_nature()
                .is_good()
        );
        assert!(
            make_eff(StatAttrModifyDimension::BasicPer, -0.5)
                .which_nature()
                .is_bad()
        );
        assert!(
            make_eff(StatAttrModifyDimension::BasicPer, 0.0)
                .which_nature()
                .is_neutral()
        );
    }

    /// 增益/减益判断：最终百分比以 0 为基线（正增量为增益）
    #[test]
    fn test_meaning_final_per() {
        assert!(
            make_eff(StatAttrModifyDimension::FinalPer, 0.5)
                .which_nature()
                .is_good()
        );
        assert!(
            make_eff(StatAttrModifyDimension::FinalPer, -0.5)
                .which_nature()
                .is_bad()
        );
        assert!(
            make_eff(StatAttrModifyDimension::FinalPer, 0.0)
                .which_nature()
                .is_neutral()
        );
    }

    /// 增益/减益判断：最终乘法以 1 为基线
    #[test]
    fn test_meaning_final_mul() {
        assert!(
            make_eff(StatAttrModifyDimension::FinalMul, 2.0)
                .which_nature()
                .is_good()
        );
        assert!(
            make_eff(StatAttrModifyDimension::FinalMul, 0.5)
                .which_nature()
                .is_bad()
        );
        assert!(
            make_eff(StatAttrModifyDimension::FinalMul, 1.0)
                .which_nature()
                .is_neutral()
        );
    }
}

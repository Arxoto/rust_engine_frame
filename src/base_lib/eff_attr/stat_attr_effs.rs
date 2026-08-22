use crate::base_lib::{
    cores::{timers::tiny_timer::HasTimer, unify_types::FixedName},
    eff_attr::{
        effects::{Effect, EffectMean, EffectMeaning},
        modifiers::{ADDITION_BASE_LINE, AggregateModifier, MULT_BASE_LINE, PERCENT_BASE_LINE},
        upsert_container::Upsert,
    },
};

/// 属性效果的类型
///
/// 计算公式见 [`StatAttrModifier::apply_modify`]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StatAttrEffType {
    /// 基础加法（描述参考：基础力量增加xx），可安全使用
    BasicAdd,
    /// 基础百分比（描述参考：基础力量提升xx%），可安全使用
    BasicPer,
    /// 百分比（描述参考：力量提升xx%），可安全使用
    FinalPer,
    /// 乘法（描述参考：力量变为原先的xx倍），指数增长、谨慎使用
    FinalMul,
}

/// 属性效果
#[derive(Clone, Debug)]
pub struct StatAttrEff<S: FixedName, Timer> {
    /// 效果类型 对应公式变量
    eff_type: StatAttrEffType,
    /// 效果
    eff: Effect<S>,
    /// 持续时间（可以不用计时器，而是计数器或者BUFF列表，通过空判断是否结束）
    duration: Timer,
}

impl<S: FixedName, Timer> StatAttrEff<S, Timer> {
    pub fn new(eff_type: StatAttrEffType, eff: Effect<S>, duration: Timer) -> Self {
        Self {
            eff_type,
            eff,
            duration,
        }
    }

    pub fn get_type(&self) -> StatAttrEffType {
        self.eff_type
    }
}

impl<S: FixedName, Timer> HasTimer for StatAttrEff<S, Timer> {
    type Timer = Timer;

    fn get_timer(&self) -> &Self::Timer {
        &self.duration
    }

    fn get_timer_mut(&mut self) -> &mut Self::Timer {
        &mut self.duration
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StatAttrEffId<S: FixedName> {
    pub eff: S,
    pub from: S,
}

impl<S: FixedName, Timer> Upsert for StatAttrEff<S, Timer> {
    type Id = StatAttrEffId<S>;

    fn gen_id(&self) -> Self::Id {
        StatAttrEffId {
            eff: self.eff.get_effect_name().clone(),
            from: self.eff.get_from_name().clone(),
        }
    }

    fn matched_id(&self, id: &Self::Id) -> bool {
        *self.eff.get_effect_name() == id.eff && *self.eff.get_from_name() == id.from
    }

    fn has_same_id(&self, other: &Self) -> bool {
        self.eff.get_effect_name() == other.eff.get_effect_name()
            && self.eff.get_from_name() == other.eff.get_from_name()
    }
}

impl<S: FixedName, Timer> EffectMeaning for StatAttrEff<S, Timer> {
    fn which_nature(&self) -> EffectMean {
        let eff_value = self.eff.get_effect_value();
        match self.eff_type {
            StatAttrEffType::BasicAdd => EffectMean::which_nature(eff_value, ADDITION_BASE_LINE),
            StatAttrEffType::BasicPer => EffectMean::which_nature(eff_value, PERCENT_BASE_LINE),
            StatAttrEffType::FinalPer => EffectMean::which_nature(eff_value, PERCENT_BASE_LINE),
            StatAttrEffType::FinalMul => EffectMean::which_nature(eff_value, MULT_BASE_LINE),
        }
    }
}

/// 属性效果修改器
///
/// 计算公式见 [`StatAttrModifier::apply_modify`]
#[derive(Debug, Default)]
pub struct StatAttrModifier(AggregateModifier);

impl StatAttrModifier {
    pub fn reduce<S: FixedName, Timer>(&mut self, eff: &StatAttrEff<S, Timer>) {
        let v = eff.eff.get_effect_value();

        match eff.eff_type {
            StatAttrEffType::BasicAdd => self.0.reduce_basic_add(v),
            StatAttrEffType::BasicPer => self.0.reduce_basic_pct(v),
            StatAttrEffType::FinalPer => self.0.reduce_final_pct(v),
            StatAttrEffType::FinalMul => self.0.reduce_final_mult(v),
        }
    }

    pub fn apply_modify(&self, v: f64) -> f64 {
        self.0.apply_modify(v)
    }
}

#[cfg(test)]
mod tests {
    use crate::base_lib::cores::timers::tick_timer::TickTimer;

    use super::*;

    /// 构造一个指定类型与数值的持久属性效果
    fn make_eff(eff_type: StatAttrEffType, eff_value: f64) -> StatAttrEff<String, TickTimer> {
        StatAttrEff::new(
            eff_type,
            Effect::new_form("from", "eff", eff_value),
            TickTimer::inf(),
        )
    }

    /// 默认修改器是恒等变换
    #[test]
    fn test_default_modifier_is_identity() {
        let am = StatAttrModifier::default();
        assert_eq!(am.apply_modify(100.0), 100.0);
        assert_eq!(am.apply_modify(0.0), 0.0);
    }

    /// 单一类型效果的累加
    #[test]
    fn test_reduce_single_type_accumulation() {
        let mut am = StatAttrModifier::default();
        am.reduce(&make_eff(StatAttrEffType::BasicAdd, 5.0));
        am.reduce(&make_eff(StatAttrEffType::BasicAdd, 3.0));
        assert_eq!(am.apply_modify(10.0), 18.0); // (8 + 10*1) * 1 * 1

        let mut am = StatAttrModifier::default();
        am.reduce(&make_eff(StatAttrEffType::BasicPer, 0.1));
        am.reduce(&make_eff(StatAttrEffType::BasicPer, 0.2));
        assert_eq!(am.apply_modify(100.0), 130.0); // (0 + 100*1.3) * 1 * 1

        let mut am = StatAttrModifier::default();
        am.reduce(&make_eff(StatAttrEffType::FinalPer, 0.5));
        assert_eq!(am.apply_modify(100.0), 150.0); // (0 + 100*1) * 1.5 * 1

        let mut am = StatAttrModifier::default();
        am.reduce(&make_eff(StatAttrEffType::FinalMul, 2.0));
        am.reduce(&make_eff(StatAttrEffType::FinalMul, 3.0));
        assert_eq!(am.apply_modify(100.0), 600.0); // (0 + 100*1) * 1 * 6
    }

    /// 混合类型按公式组合
    #[test]
    fn test_reduce_mixed_formula() {
        let mut am = StatAttrModifier::default();
        am.reduce(&make_eff(StatAttrEffType::BasicAdd, 10.0));
        am.reduce(&make_eff(StatAttrEffType::BasicPer, 0.5));
        am.reduce(&make_eff(StatAttrEffType::FinalPer, 0.25));
        am.reduce(&make_eff(StatAttrEffType::FinalMul, 2.0));
        // (10 + 100*1.5) * 1.25 * 2 = 400
        assert_eq!(am.apply_modify(100.0), 400.0);
    }

    /// 边界值：负基础百分比削减基础值
    #[test]
    fn test_basic_per_negative_reduces() {
        let mut am = StatAttrModifier::default();
        am.reduce(&make_eff(StatAttrEffType::BasicPer, -0.5));
        assert_eq!(am.apply_modify(100.0), 50.0);
    }

    /// 边界值：负基础加值使结果低于基础值
    #[test]
    fn test_basic_add_negative() {
        let mut am = StatAttrModifier::default();
        am.reduce(&make_eff(StatAttrEffType::BasicAdd, -30.0));
        assert_eq!(am.apply_modify(100.0), 70.0);
    }

    /// 边界值：最终乘法为 0 时结果恒为 0
    #[test]
    fn test_final_mul_zero() {
        let mut am = StatAttrModifier::default();
        am.reduce(&make_eff(StatAttrEffType::FinalMul, 0.0));
        assert_eq!(am.apply_modify(100.0), 0.0);
    }

    /// 边界值：最终乘法为负时符号翻转
    #[test]
    fn test_final_mul_negative_flips_sign() {
        let mut am = StatAttrModifier::default();
        am.reduce(&make_eff(StatAttrEffType::FinalMul, -2.0));
        assert_eq!(am.apply_modify(100.0), -200.0);
    }

    /// 边界值：基础值为 0 时结果只取决于基础加法
    #[test]
    fn test_zero_origin() {
        let mut am = StatAttrModifier::default();
        am.reduce(&make_eff(StatAttrEffType::BasicPer, 2.0));
        am.reduce(&make_eff(StatAttrEffType::FinalMul, 3.0));
        assert_eq!(am.apply_modify(0.0), 0.0);
    }

    /// 增益/减益判断：基础加法以 0 为基线
    #[test]
    fn test_meaning_basic_add() {
        assert!(
            make_eff(StatAttrEffType::BasicAdd, 5.0)
                .which_nature()
                .is_good()
        );
        assert!(
            make_eff(StatAttrEffType::BasicAdd, -5.0)
                .which_nature()
                .is_bad()
        );
        assert!(
            make_eff(StatAttrEffType::BasicAdd, 0.0)
                .which_nature()
                .is_neutral()
        );
    }

    /// 增益/减益判断：基础百分比以 0 为基线（正增量为增益）
    #[test]
    fn test_meaning_basic_per() {
        assert!(
            make_eff(StatAttrEffType::BasicPer, 0.5)
                .which_nature()
                .is_good()
        );
        assert!(
            make_eff(StatAttrEffType::BasicPer, -0.5)
                .which_nature()
                .is_bad()
        );
        assert!(
            make_eff(StatAttrEffType::BasicPer, 0.0)
                .which_nature()
                .is_neutral()
        );
    }

    /// 增益/减益判断：最终百分比以 0 为基线（正增量为增益）
    #[test]
    fn test_meaning_final_per() {
        assert!(
            make_eff(StatAttrEffType::FinalPer, 0.5)
                .which_nature()
                .is_good()
        );
        assert!(
            make_eff(StatAttrEffType::FinalPer, -0.5)
                .which_nature()
                .is_bad()
        );
        assert!(
            make_eff(StatAttrEffType::FinalPer, 0.0)
                .which_nature()
                .is_neutral()
        );
    }

    /// 增益/减益判断：最终乘法以 1 为基线
    #[test]
    fn test_meaning_final_mul() {
        assert!(
            make_eff(StatAttrEffType::FinalMul, 2.0)
                .which_nature()
                .is_good()
        );
        assert!(
            make_eff(StatAttrEffType::FinalMul, 0.5)
                .which_nature()
                .is_bad()
        );
        assert!(
            make_eff(StatAttrEffType::FinalMul, 1.0)
                .which_nature()
                .is_neutral()
        );
    }

}

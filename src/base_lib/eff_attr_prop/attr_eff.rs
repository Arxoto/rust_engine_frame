use crate::base_lib::{
    cores::{timers::tiny_timer::HasTimer, unify_types::FixedName},
    eff_attr_prop::{
        effects::{Effect, EffectMean, EffectMeaning},
        upsert_container::Upsert,
    },
};

/// Attr 属性效果的类型
///
/// 计算公式见 [`AttrModifier::apply_modify`]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttrEffectType {
    /// 基础加法（描述参考：基础伤害增加xx），可安全使用
    BasicAdd,
    /// 基础百分比（描述参考：基础伤害提升xx%），可安全使用
    BasicPer,
    /// 最终百分比（描述参考：伤害提升xx%），可安全使用
    FinalPer,
    /// 最终乘法（描述参考：造成xx倍伤害），指数增长、谨慎使用
    FinalMul,
}

/// Attr 属性效果
///
/// 表示为：力量、攻击力等面板属性
#[derive(Clone, Debug)]
pub struct AttrEffect<S: FixedName, Timer> {
    /// 效果类型 对应公式变量
    eff_type: AttrEffectType,
    /// 效果
    eff: Effect<S>,
    /// 持续时间（可以不用计时器，而是计数器或者BUFF列表，通过空判断是否结束）
    duration: Timer,
}

impl<S: FixedName, Timer> AttrEffect<S, Timer> {
    pub fn new(eff_type: AttrEffectType, eff: Effect<S>, duration: Timer) -> Self {
        Self {
            eff_type,
            eff,
            duration,
        }
    }

    pub fn get_type(&self) -> AttrEffectType {
        self.eff_type
    }
}

impl<S: FixedName, Timer> HasTimer for AttrEffect<S, Timer> {
    type Timer = Timer;

    fn get_timer(&self) -> &Self::Timer {
        &self.duration
    }

    fn get_timer_mut(&mut self) -> &mut Self::Timer {
        &mut self.duration
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AttrEffId<S: FixedName> {
    pub eff: S,
    pub from: S,
}

impl<S: FixedName, Timer> Upsert for AttrEffect<S, Timer> {
    type Id = AttrEffId<S>;

    fn gen_id(&self) -> Self::Id {
        AttrEffId {
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

impl<S: FixedName, Timer> EffectMeaning for AttrEffect<S, Timer> {
    fn which_nature(&self) -> EffectMean {
        let eff_value = self.eff.get_effect_value();
        match self.eff_type {
            AttrEffectType::BasicAdd => EffectMean::which_nature(eff_value, BASIC_ADD_BASE_LINE),
            AttrEffectType::BasicPer => EffectMean::which_nature(eff_value, BASIC_PER_BASE_LINE),
            AttrEffectType::FinalPer => EffectMean::which_nature(eff_value, FINAL_PER_BASE_LINE),
            AttrEffectType::FinalMul => EffectMean::which_nature(eff_value, FINAL_MUL_BASE_LINE),
        }
    }
}

const BASIC_ADD_BASE_LINE: f64 = 0.0;
const BASIC_PER_BASE_LINE: f64 = 0.0;
const FINAL_PER_BASE_LINE: f64 = 0.0;
const FINAL_MUL_BASE_LINE: f64 = 1.0;

/// Attr属性效果修改器
///
/// 计算公式见 [`AttrModifier::apply_modify`]
pub struct AttrModifier {
    basic_add: f64,
    basic_per: f64,
    final_per: f64,
    final_mul: f64,
}

impl Default for AttrModifier {
    fn default() -> Self {
        Self {
            basic_add: BASIC_ADD_BASE_LINE,
            basic_per: BASIC_PER_BASE_LINE,
            final_per: FINAL_PER_BASE_LINE,
            final_mul: FINAL_MUL_BASE_LINE,
        }
    }
}

impl AttrModifier {
    pub fn reduce<S: FixedName, Timer>(&mut self, eff: &AttrEffect<S, Timer>) {
        let eff_value = eff.eff.get_effect_value();

        match eff.eff_type {
            AttrEffectType::BasicAdd => self.basic_add += eff_value,
            AttrEffectType::BasicPer => self.basic_per += eff_value,
            AttrEffectType::FinalPer => self.final_per += eff_value,
            AttrEffectType::FinalMul => self.final_mul *= eff_value,
        }
    }

    /// 计算公式 `(b_add + base_value * (1 + b_per)) * (1 + f_per) * f_multi`
    ///
    /// 加法类基线为 0 （`1 + base`计算），乘法类基线为 1
    pub fn apply_modify(&self, v: f64) -> f64 {
        (self.basic_add + v * (1.0 + self.basic_per)) * (1.0 + self.final_per) * self.final_mul
    }
}

#[cfg(test)]
mod tests {
    use crate::base_lib::cores::timers::tick_timer::TickTimer;

    use super::*;

    /// 构造一个指定类型与数值的持久属性效果
    fn make_eff(eff_type: AttrEffectType, eff_value: f64) -> AttrEffect<String, TickTimer> {
        AttrEffect::new(
            eff_type,
            Effect::new_form("from", "eff", eff_value),
            TickTimer::inf(),
        )
    }

    /// 默认修改器是恒等变换
    #[test]
    fn test_default_modifier_is_identity() {
        let am = AttrModifier::default();
        assert_eq!(am.apply_modify(100.0), 100.0);
        assert_eq!(am.apply_modify(0.0), 0.0);
    }

    /// 单一类型效果的累加
    #[test]
    fn test_reduce_single_type_accumulation() {
        let mut am = AttrModifier::default();
        am.reduce(&make_eff(AttrEffectType::BasicAdd, 5.0));
        am.reduce(&make_eff(AttrEffectType::BasicAdd, 3.0));
        assert_eq!(am.apply_modify(10.0), 18.0); // (8 + 10*1) * 1 * 1

        let mut am = AttrModifier::default();
        am.reduce(&make_eff(AttrEffectType::BasicPer, 0.1));
        am.reduce(&make_eff(AttrEffectType::BasicPer, 0.2));
        assert_eq!(am.apply_modify(100.0), 130.0); // (0 + 100*1.3) * 1 * 1

        let mut am = AttrModifier::default();
        am.reduce(&make_eff(AttrEffectType::FinalPer, 0.5));
        assert_eq!(am.apply_modify(100.0), 150.0); // (0 + 100*1) * 1.5 * 1

        let mut am = AttrModifier::default();
        am.reduce(&make_eff(AttrEffectType::FinalMul, 2.0));
        am.reduce(&make_eff(AttrEffectType::FinalMul, 3.0));
        assert_eq!(am.apply_modify(100.0), 600.0); // (0 + 100*1) * 1 * 6
    }

    /// 混合类型按公式组合
    #[test]
    fn test_reduce_mixed_formula() {
        let mut am = AttrModifier::default();
        am.reduce(&make_eff(AttrEffectType::BasicAdd, 10.0));
        am.reduce(&make_eff(AttrEffectType::BasicPer, 0.5));
        am.reduce(&make_eff(AttrEffectType::FinalPer, 0.25));
        am.reduce(&make_eff(AttrEffectType::FinalMul, 2.0));
        // (10 + 100*1.5) * 1.25 * 2 = 400
        assert_eq!(am.apply_modify(100.0), 400.0);
    }

    /// 边界值：负基础百分比削减基础值
    #[test]
    fn test_basic_per_negative_reduces() {
        let mut am = AttrModifier::default();
        am.reduce(&make_eff(AttrEffectType::BasicPer, -0.5));
        assert_eq!(am.apply_modify(100.0), 50.0);
    }

    /// 边界值：负基础加值使结果低于基础值
    #[test]
    fn test_basic_add_negative() {
        let mut am = AttrModifier::default();
        am.reduce(&make_eff(AttrEffectType::BasicAdd, -30.0));
        assert_eq!(am.apply_modify(100.0), 70.0);
    }

    /// 边界值：最终乘法为 0 时结果恒为 0
    #[test]
    fn test_final_mul_zero() {
        let mut am = AttrModifier::default();
        am.reduce(&make_eff(AttrEffectType::FinalMul, 0.0));
        assert_eq!(am.apply_modify(100.0), 0.0);
    }

    /// 边界值：最终乘法为负时符号翻转
    #[test]
    fn test_final_mul_negative_flips_sign() {
        let mut am = AttrModifier::default();
        am.reduce(&make_eff(AttrEffectType::FinalMul, -2.0));
        assert_eq!(am.apply_modify(100.0), -200.0);
    }

    /// 边界值：基础值为 0 时结果只取决于基础加法
    #[test]
    fn test_zero_origin() {
        let mut am = AttrModifier::default();
        am.reduce(&make_eff(AttrEffectType::BasicPer, 2.0));
        am.reduce(&make_eff(AttrEffectType::FinalMul, 3.0));
        assert_eq!(am.apply_modify(0.0), 0.0);
    }

    /// 增益/减益判断：基础加法以 0 为基线
    #[test]
    fn test_meaning_basic_add() {
        assert!(
            make_eff(AttrEffectType::BasicAdd, 5.0)
                .which_nature()
                .is_good()
        );
        assert!(
            make_eff(AttrEffectType::BasicAdd, -5.0)
                .which_nature()
                .is_bad()
        );
        assert!(
            make_eff(AttrEffectType::BasicAdd, 0.0)
                .which_nature()
                .is_neutral()
        );
    }

    /// 增益/减益判断：基础百分比以 0 为基线（正增量为增益）
    #[test]
    fn test_meaning_basic_per() {
        assert!(
            make_eff(AttrEffectType::BasicPer, 0.5)
                .which_nature()
                .is_good()
        );
        assert!(
            make_eff(AttrEffectType::BasicPer, -0.5)
                .which_nature()
                .is_bad()
        );
        assert!(
            make_eff(AttrEffectType::BasicPer, 0.0)
                .which_nature()
                .is_neutral()
        );
    }

    /// 增益/减益判断：最终百分比以 0 为基线（正增量为增益）
    #[test]
    fn test_meaning_final_per() {
        assert!(
            make_eff(AttrEffectType::FinalPer, 0.5)
                .which_nature()
                .is_good()
        );
        assert!(
            make_eff(AttrEffectType::FinalPer, -0.5)
                .which_nature()
                .is_bad()
        );
        assert!(
            make_eff(AttrEffectType::FinalPer, 0.0)
                .which_nature()
                .is_neutral()
        );
    }

    /// 增益/减益判断：最终乘法以 1 为基线
    #[test]
    fn test_meaning_final_mul() {
        assert!(
            make_eff(AttrEffectType::FinalMul, 2.0)
                .which_nature()
                .is_good()
        );
        assert!(
            make_eff(AttrEffectType::FinalMul, 0.5)
                .which_nature()
                .is_bad()
        );
        assert!(
            make_eff(AttrEffectType::FinalMul, 1.0)
                .which_nature()
                .is_neutral()
        );
    }

    /// Upsert 幂等匹配：同来源同效果名视为同 id
    #[test]
    fn test_upsert_same_id() {
        let a = make_eff(AttrEffectType::BasicAdd, 1.0);
        let b = make_eff(AttrEffectType::BasicAdd, 2.0);
        assert!(a.has_same_id(&b));
        assert!(b.has_same_id(&a));

        let id = a.gen_id();
        assert!(a.matched_id(&id));
        assert!(b.matched_id(&id));
    }

    /// Upsert 匹配：效果名不同则不同 id
    #[test]
    fn test_upsert_diff_eff_name() {
        let a: AttrEffect<String, TickTimer> = AttrEffect::new(
            AttrEffectType::BasicAdd,
            Effect::new_form("from", "eff_a", 1.0),
            TickTimer::inf(),
        );
        let b: AttrEffect<String, TickTimer> = AttrEffect::new(
            AttrEffectType::BasicAdd,
            Effect::new_form("from", "eff_b", 1.0),
            TickTimer::inf(),
        );
        assert!(!a.has_same_id(&b));
    }

    /// Upsert 匹配：来源不同则不同 id
    #[test]
    fn test_upsert_diff_from_name() {
        let a: AttrEffect<String, TickTimer> = AttrEffect::new(
            AttrEffectType::BasicAdd,
            Effect::new_form("from_a", "eff", 1.0),
            TickTimer::inf(),
        );
        let b: AttrEffect<String, TickTimer> = AttrEffect::new(
            AttrEffectType::BasicAdd,
            Effect::new_form("from_b", "eff", 1.0),
            TickTimer::inf(),
        );
        assert!(!a.has_same_id(&b));
    }
}

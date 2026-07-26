use crate::base_lib::{
    cores::design_patterns::ContextWrapper,
    eff_attr_prop::effects::{Effect, EffectMean, EffectMeaning},
};

/// Attr 属性效果的类型
///
/// 计算公式见 [`AttrModifier::apply_modify`]
#[derive(Clone, Copy, Debug)]
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
pub struct AttrEffect<S, Timer> {
    pub eff_type: AttrEffectType,
    pub effect: Effect<S>,
    pub duration: Timer,
}

impl<S> EffectMeaning for ContextWrapper<&AttrEffectType, &Effect<S>> {
    fn which_nature(&self) -> EffectMean {
        let eff_value = self.ctx.get_effect_value();
        match self.inner {
            AttrEffectType::BasicAdd => EffectMean::which_nature(eff_value, BASIC_ADD_BASE_LINE),
            AttrEffectType::BasicPer => EffectMean::which_nature(eff_value, BASIC_PER_BASE_LINE),
            AttrEffectType::FinalPer => EffectMean::which_nature(eff_value, FINAL_PER_BASE_LINE),
            AttrEffectType::FinalMul => EffectMean::which_nature(eff_value, FINAL_MUL_BASE_LINE),
        }
    }
}

const BASIC_ADD_BASE_LINE: f64 = 0.0;
const BASIC_PER_BASE_LINE: f64 = 1.0;
const FINAL_PER_BASE_LINE: f64 = 1.0;
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
    pub fn reduce<S, Timer>(&mut self, eff: &AttrEffect<S, Timer>) {
        let eff_value = eff.effect.get_effect_value();
        let eff_stack = eff.effect.get_stack_value();

        match eff.eff_type {
            AttrEffectType::BasicAdd => self.basic_add += eff_value * eff_stack as f64,
            AttrEffectType::BasicPer => self.basic_per += eff_value * eff_stack as f64,
            AttrEffectType::FinalPer => self.final_per += eff_value * eff_stack as f64,
            AttrEffectType::FinalMul => self.final_mul *= eff_value.powi(eff_stack),
        }
    }

    /// 计算公式 `(b_add + base_value * b_per) * f_per * f_multi`
    pub fn apply_modify(&self, v: f64) -> f64 {
        (self.basic_add + v * self.basic_per) * self.final_per * self.final_mul
    }
}

// todo test 基础功能 边界值

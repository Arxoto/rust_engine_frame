use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr_prop::{
        eff_container::WithId,
        effects::{Effect, EffectMean, EffectMeaning, EffectStackable},
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
    eff_id: (AttrEffectType, S),
    eff_type: AttrEffectType,
    effect: Effect<S>,
    duration: Timer,
}

impl<S: FixedName, Timer> AttrEffect<S, Timer> {
    pub fn new(eff_type: AttrEffectType, effect: Effect<S>, duration: Timer) -> Self {
        Self {
            eff_id: (eff_type, effect.get_effect_name().clone()),
            eff_type,
            effect,
            duration,
        }
    }
}

impl<S: FixedName, Timer> WithId for AttrEffect<S, Timer> {
    type Id = (AttrEffectType, S);

    fn get_id(&self) -> &Self::Id {
        // todo 确认能否使用内部引用直接返回，节省空间
        &self.eff_id
    }
}

impl<S: FixedName, Timer> EffectStackable for AttrEffect<S, Timer> {
    fn do_stack(&mut self, other: &Self) {
        // todo 把几个常用的堆叠逻辑复用，不同的属性效果可能会有不同的堆叠逻辑

        // 不可堆叠 只能替换
        let source = &mut self.effect;
        let target = &other.effect;

        source.set_from_name(target.get_from_name().clone());
        source.update_eff_val_by(target);
    }
}

impl<S: FixedName, Timer> EffectMeaning for AttrEffect<S, Timer> {
    fn which_nature(&self) -> EffectMean {
        let eff_value = self.effect.get_effect_value();
        match self.eff_type {
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
    pub fn reduce<S: FixedName, Timer>(&mut self, eff: &AttrEffect<S, Timer>) {
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

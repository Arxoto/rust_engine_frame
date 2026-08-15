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
        let eff_value = eff.eff.get_effect_value();

        match eff.eff_type {
            AttrEffectType::BasicAdd => self.basic_add += eff_value,
            AttrEffectType::BasicPer => self.basic_per += eff_value,
            AttrEffectType::FinalPer => self.final_per += eff_value,
            AttrEffectType::FinalMul => self.final_mul *= eff_value,
        }
    }

    /// 计算公式 `(b_add + base_value * b_per) * f_per * f_multi`
    pub fn apply_modify(&self, v: f64) -> f64 {
        (self.basic_add + v * self.basic_per) * self.final_per * self.final_mul
    }
}

// todo test 基础功能 边界值

use crate::base_lib::{
    cores::{timers::tiny_timer::HasTimer, unify_types::FixedName},
    eff_attr::{
        effects::{EffId, EffIdRef, Effect, EffectMean, EffectMeaning},
        modifiers::{ADDITION_BASE_LINE, AnchorModifier, PERCENT_BASE_LINE},
        upsert_container::Upsert,
    },
};

/// 属性效果的类型
///
/// 计算公式见 [`BoundAttrModifier::apply_modify`]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BoundAttrEffType {
    /// 基础加法（描述参考：基础力量增加xx），可安全使用
    BasicAdd,
    /// 基础百分比（描述参考：基础力量提升xx%），可安全使用
    BasicPer,
}

impl BoundAttrEffType {
    pub fn calc_real_val(&self, base: f64, eff_val: f64) -> f64 {
        match self {
            BoundAttrEffType::BasicAdd => eff_val,
            BoundAttrEffType::BasicPer => base * eff_val,
        }
    }
}

/// 属性效果
///
/// 若想在修改上限的同时修改实际值，那么需要同时生成【修改上限】的效果和【修改实际值】的效果
///
/// 为了保证两者修改效果一致，限制修改维度只能基于基础值修改（不会被放大缩小产生偏差）
#[derive(Clone, Debug)]
pub struct BoundAttrEff<S: FixedName, Timer> {
    /// 效果类型 对应公式变量
    eff_type: BoundAttrEffType,
    /// 效果
    eff: Effect<S>,
    /// 持续时间（可以不用计时器，而是计数器或者BUFF列表，通过空判断是否结束）
    duration: Timer,
}

impl<S: FixedName, Timer> BoundAttrEff<S, Timer> {
    pub fn new(eff_type: BoundAttrEffType, eff: Effect<S>, duration: Timer) -> Self {
        Self {
            eff_type,
            eff,
            duration,
        }
    }

    pub fn get_type(&self) -> BoundAttrEffType {
        self.eff_type
    }
}

impl<S: FixedName, Timer> HasTimer for BoundAttrEff<S, Timer> {
    type Timer = Timer;

    fn get_timer(&self) -> &Self::Timer {
        &self.duration
    }

    fn get_timer_mut(&mut self) -> &mut Self::Timer {
        &mut self.duration
    }
}

impl<S: FixedName, Timer> Upsert for BoundAttrEff<S, Timer> {
    type Id = EffId<S>;
    type IdRef<'a>
        = EffIdRef<'a, S>
    where
        Self: 'a;

    fn gen_id(&self) -> Self::Id {
        self.eff.gen_id()
    }

    fn id_ref<'a>(&'a self) -> Self::IdRef<'a> {
        self.eff.id_ref()
    }
}

impl<S: FixedName, Timer> EffectMeaning for BoundAttrEff<S, Timer> {
    fn which_nature(&self) -> EffectMean {
        let eff_value = self.eff.get_effect_value();
        match self.eff_type {
            BoundAttrEffType::BasicAdd => EffectMean::which_nature(eff_value, ADDITION_BASE_LINE),
            BoundAttrEffType::BasicPer => EffectMean::which_nature(eff_value, PERCENT_BASE_LINE),
        }
    }
}

/// 属性效果修改器
///
/// 计算公式见 [`BoundAttrModifier::apply_modify`]
#[derive(Debug, Default)]
pub struct BoundAttrModifier(AnchorModifier);

impl BoundAttrModifier {
    pub fn reduce<S: FixedName, Timer>(&mut self, eff: &BoundAttrEff<S, Timer>) {
        let v = eff.eff.get_effect_value();

        match eff.eff_type {
            BoundAttrEffType::BasicAdd => self.0.reduce_add(v),
            BoundAttrEffType::BasicPer => self.0.reduce_pct(v),
        }
    }

    pub fn apply_modify(&self, v: f64) -> f64 {
        self.0.apply_modify(v)
    }
}

// todo test

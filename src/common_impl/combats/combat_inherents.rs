//! 内禀属性
//!
//! 字段设为 pub 支持拆分平铺到实体中去

use crate::base_lib::{
    cores::{timers::static_timer::StaticTimer, unify_types::FixedName},
    eff_attr::{
        stat_attr_effs::StatAttrEff, stat_attrs::StatAttr, upsert_container::UpsertContainer,
    },
};

/// 气力
pub struct Strength(pub StatAttr);

/// 信念
pub struct Belief(pub StatAttr);

pub struct StrengthEffs<S: FixedName>(pub UpsertContainer<StatAttrEff<S, StaticTimer>>);
pub struct BeliefEffs<S: FixedName>(pub UpsertContainer<StatAttrEff<S, StaticTimer>>);

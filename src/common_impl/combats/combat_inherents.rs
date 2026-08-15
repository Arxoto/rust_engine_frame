//! 内禀属性
//!
//! 字段设为 pub 支持拆分平铺到实体中去

use crate::base_lib::{
    cores::{timers::static_timer::StaticTimer, unify_types::FixedName},
    eff_attr_prop::{attr_eff::AttrEffect, attrs::Attr, upsert_container::UpsertContainer},
};

/// 气力
pub struct Strength(pub Attr);

/// 信念
pub struct Belief(pub Attr);

pub struct StrengthEffs<S: FixedName>(pub UpsertContainer<AttrEffect<S, StaticTimer>>);
pub struct BeliefEffs<S: FixedName>(pub UpsertContainer<AttrEffect<S, StaticTimer>>);

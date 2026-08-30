//! 内禀属性
//!
//! 字段设为 pub 支持拆分平铺到实体中去

use crate::base_lib::eff_attr::{
    modifier_collections::ModifierCollection, stat_attr_modifiers::StatAttrModifier,
    stat_attrs::StatAttr,
};

/// 气力
pub struct Strength(pub StatAttr);

/// 信念
pub struct Belief(pub StatAttr);

pub struct StrengthEffs(pub ModifierCollection<StatAttrModifier>);
pub struct BeliefEffs(pub ModifierCollection<StatAttrModifier>);

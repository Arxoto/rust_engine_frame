//! 外赋属性
//!
//! 字段设为 pub 支持拆分平铺到实体中去

use crate::base_lib::eff_attr::{
    modifier_collections::ModifierCollection, stat_attr_modifiers::StatAttrModifier,
    stat_attrs::StatAttr,
};

/// 武器锋利度
pub struct WeaponSharp(pub StatAttr);

/// 武器质量
pub struct WeaponMass(pub StatAttr);

/// 盔甲坚韧
pub struct ArmorHard(pub StatAttr);

/// 盔甲柔韧
pub struct ArmorSoft(pub StatAttr);

/// 盔甲质量
pub struct ArmorMass(pub StatAttr);

pub struct WeaponSharpEffs(pub ModifierCollection<StatAttrModifier>);
pub struct WeaponMassEffs(pub ModifierCollection<StatAttrModifier>);
pub struct ArmorHardEffs(pub ModifierCollection<StatAttrModifier>);
pub struct ArmorSoftEffs(pub ModifierCollection<StatAttrModifier>);
pub struct ArmorMassEffs(pub ModifierCollection<StatAttrModifier>);

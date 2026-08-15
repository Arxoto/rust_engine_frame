//! 外赋属性
//!
//! 字段设为 pub 支持拆分平铺到实体中去

use crate::base_lib::{
    cores::{timers::static_timer::StaticTimer, unify_types::FixedName},
    eff_attr_prop::{attr_eff::AttrEffect, attrs::Attr, upsert_container::UpsertContainer},
};

/// 武器锋利度
pub struct WeaponSharp(pub Attr);

/// 武器质量
pub struct WeaponMass(pub Attr);

/// 盔甲坚韧
pub struct ArmorHard(pub Attr);

/// 盔甲柔韧
pub struct ArmorSoft(pub Attr);

/// 盔甲质量
pub struct ArmorMass(pub Attr);

pub struct WeaponSharpEffs<S: FixedName>(pub UpsertContainer<AttrEffect<S, StaticTimer>>);
pub struct WeaponMassEffs<S: FixedName>(pub UpsertContainer<AttrEffect<S, StaticTimer>>);
pub struct ArmorHardEffs<S: FixedName>(pub UpsertContainer<AttrEffect<S, StaticTimer>>);
pub struct ArmorSoftEffs<S: FixedName>(pub UpsertContainer<AttrEffect<S, StaticTimer>>);
pub struct ArmorMassEffs<S: FixedName>(pub UpsertContainer<AttrEffect<S, StaticTimer>>);

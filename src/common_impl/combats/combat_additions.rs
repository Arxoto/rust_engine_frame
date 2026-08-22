//! 外赋属性
//!
//! 字段设为 pub 支持拆分平铺到实体中去

use crate::base_lib::{
    cores::{timers::static_timer::StaticTimer, unify_types::FixedName},
    eff_attr::{
        stat_attr_effs::StatAttrEff, stat_attrs::StatAttr, upsert_container::UpsertContainer,
    },
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

pub struct WeaponSharpEffs<S: FixedName>(pub UpsertContainer<StatAttrEff<S, StaticTimer>>);
pub struct WeaponMassEffs<S: FixedName>(pub UpsertContainer<StatAttrEff<S, StaticTimer>>);
pub struct ArmorHardEffs<S: FixedName>(pub UpsertContainer<StatAttrEff<S, StaticTimer>>);
pub struct ArmorSoftEffs<S: FixedName>(pub UpsertContainer<StatAttrEff<S, StaticTimer>>);
pub struct ArmorMassEffs<S: FixedName>(pub UpsertContainer<StatAttrEff<S, StaticTimer>>);

//! 对 [`BoundedAttr`] 的通用修改描述与修改值计算

use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr::{
        bound_attr_modifiers::{BoundAttrModifier, BoundAttrModifyDimension},
        bound_attrs::BoundAttr,
        bounded_attrs::BoundedAttr,
        effects::Effect,
        modifier_collections::ModifiableAttr,
    },
};

/// 对 [`BoundedAttr`] 的修改计算方式
#[derive(Debug, Clone, Copy)]
pub enum AttrAlterEffType {
    /// 绝对值修改
    Val,
    /// 根据当前值的百分比
    CurPer,
    /// 根据最大值的百分比
    MaxPer,
}

impl AttrAlterEffType {
    /// 根据效果类型计算绝对值
    ///
    /// 为了内聚 逻辑必须在这里实现 因此需要传入参数
    pub fn calc_alter_val(
        &self,
        eff_val: f64,
        bounded_attr: &BoundedAttr,
        upper_bound: &BoundAttr,
    ) -> f64 {
        match self {
            Self::Val => eff_val,
            Self::CurPer => eff_val * bounded_attr.get_snapshot_value(),
            Self::MaxPer => eff_val * upper_bound.get_current(),
        }
    }
}

/// 对 [`BoundedAttr`] 的 (Instant) 修改效果: 计算方式 + 效果
#[derive(Debug, Clone)]
pub struct AttrAlterEff<S: FixedName> {
    eff_type: AttrAlterEffType,
    eff: Effect<S>,
}

impl<S: FixedName> AttrAlterEff<S> {
    /// 构造修改效果
    pub fn new(eff_type: AttrAlterEffType, eff: Effect<S>) -> Self {
        Self { eff_type, eff }
    }

    pub fn get_type(&self) -> AttrAlterEffType {
        self.eff_type
    }

    pub fn take_eff(self) -> Effect<S> {
        self.eff
    }

    /// 计算 [`AttrAlterEff`] 的绝对值
    pub fn calc_alter_val(&self, bounded_attr: &BoundedAttr, upper_bound: &BoundAttr) -> f64 {
        let eff_val = self.eff.get_effect_value();
        self.eff_type
            .calc_alter_val(eff_val, bounded_attr, upper_bound)
    }

    /// 基于属性上限的修改效果，生成数值一致的 [`BoundAttrModifier`] [`AttrAlterEff`] 效果
    ///
    /// 若是针对下限生效的效果，则应该通过下限钳制自动修正
    pub fn gen_effs_for_upper_bound(
        upper_bound: &BoundAttr,
        eff_type: BoundAttrModifyDimension,
        mut effect: Effect<S>,
    ) -> (BoundAttrModifier, Self) {
        let eff_val = eff_type.calc_real_val(upper_bound.get_current(), effect.get_effect_value());
        effect.set_effect_value(eff_val);

        Self::gen_effs_for_upper_bound_by_val(effect)
    }

    pub fn gen_effs_for_upper_bound_by_val(effect: Effect<S>) -> (BoundAttrModifier, Self) {
        let dimension = BoundAttrModifyDimension::BasicAdd;
        let bound_attr_modifier = BoundAttrModifier::new(dimension, effect.get_effect_value());

        let eff_type = AttrAlterEffType::Val;
        let attr_alter_eff = Self::new(eff_type, effect);

        (bound_attr_modifier, attr_alter_eff)
    }
}

// todo test

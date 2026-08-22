use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr::bound_attr_effs::{BoundAttrEff, BoundAttrModifier},
};

/// 边界约束属性，用于限制某属性的上下界限
///
/// 对应效果的计算只能基于基础值，因此修改值可预测，可用作各种公式计算的锚点
/// - 如：“增加血量上限的同时增加等量的血量”
#[derive(Debug)]
pub struct BoundAttr {
    /// 原始值，未经过修改器修改
    origin: f64,
    /// 当前值，经过修改器修改
    current: f64,
}

impl BoundAttr {
    pub fn new(origin: f64) -> Self {
        Self {
            origin,
            current: origin,
        }
    }

    pub fn get_origin(&self) -> f64 {
        self.origin
    }

    pub fn get_current(&self) -> f64 {
        self.current
    }

    /// 刷新属性，在效果更新后
    pub fn refresh_value<'a, S: FixedName + 'a, Timer: 'a>(
        &mut self,
        effs: impl Iterator<Item = &'a BoundAttrEff<S, Timer>>,
    ) {
        let mut modifier = BoundAttrModifier::default();

        for ele in effs {
            modifier.reduce(ele);
        }

        self.current = modifier.apply_modify(self.origin);
    }
}

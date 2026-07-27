use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr_prop::upsert_container::{Upsert, UpsertContainer},
};

/// attribute 属性 一般用作角色属性值 可被效果影响
///
/// 另一种风格是将所有基础属性展开平铺，每个效果直接修改对应属性，灵活但复杂
#[derive(Debug, Default)]
pub struct Attr {
    origin: f64,
    current: f64,
}

impl Attr {
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

    pub fn refresh_value<E: Upsert>(&mut self, _effs: UpsertContainer<E>) {
        // todo
    }
}

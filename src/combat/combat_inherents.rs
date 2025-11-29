use crate::{attrs::dyn_attr::DynAttr, cores::unify_type::FixedName};

/// 内禀属性
pub struct CombatInherentAttr<S: FixedName> {
    /// 气力
    pub(crate) strength: DynAttr<S>,
    /// 信念
    pub(crate) belief: DynAttr<S>,
}

impl<S: FixedName> CombatInherentAttr<S> {
    pub fn new(strength: f64, belief: f64) -> CombatInherentAttr<S> {
        CombatInherentAttr {
            strength: DynAttr::new(strength),
            belief: DynAttr::new(belief),
        }
    }
}

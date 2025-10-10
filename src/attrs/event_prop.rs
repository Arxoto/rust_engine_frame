//! 动态属性 property 的响应事件类型

use crate::{cores::unify_type::FixedName, effects::native_effect::Effect};

#[derive(Debug, Default)]
pub struct DynPropAlterResult {
    /// 效果实际修改值
    pub(crate) delta: f64,
}

impl DynPropAlterResult {
    /// 有益的
    pub fn is_beneficial(&self) -> bool {
        self.delta > 0.0
    }

    /// 有害的
    pub fn is_harmful(&self) -> bool {
        self.delta < 0.0
    }
}

pub struct DynPropProcessResult<S: FixedName> {
    /// 被哪个效果作用后达到最小值
    pub to_min_by: Option<Effect<S>>, // pub-external
}

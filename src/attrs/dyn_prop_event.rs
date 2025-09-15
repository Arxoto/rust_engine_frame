//! 动态属性 property 的响应事件类型

use crate::{cores::unify_type::FixedName, effects::native_effect::Effect};

pub trait OnceCurMin<S: FixedName>: FnMut(&Effect<S>) {}
impl<S: FixedName, T: FnMut(&Effect<S>)> OnceCurMin<S> for T {}

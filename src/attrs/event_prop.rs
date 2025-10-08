//! 动态属性 property 的响应事件类型

use crate::{cores::unify_type::FixedName, effects::native_effect::Effect};

pub trait DynPropEvents<S: FixedName> {
    fn once_current_to_min(&mut self, eff: &Effect<S>);
}

/// 字段其实应尽量不用 Option 比较好
/// 
/// 期望中，仅单元测试中可能为 None ，生产环境均为 Some
/// 
/// 但是，实现 Default 时无法对闭包做默认实现，只能期望编译期自动优化
#[derive(Clone, Debug)]
pub struct DynPropEventsImpl<S, FnCurMin>
where
    S: FixedName,
    FnCurMin: FnMut(&Effect<S>),
{
    // 让编译器以为使用了该泛型 零成本
    pub(crate) _marker: std::marker::PhantomData<S>,

    pub(crate) once_current_to_min: Option<FnCurMin>,
}

impl<S, FnCurMin> DynPropEvents<S> for DynPropEventsImpl<S, FnCurMin>
where
    S: FixedName,
    FnCurMin: FnMut(&Effect<S>),
{
    fn once_current_to_min(&mut self, eff: &Effect<S>) {
        if let Some(notify) = &mut self.once_current_to_min {
            notify(eff);
        }
    }
}

// just for test
impl<S, FnCurMin> Default for DynPropEventsImpl<S, FnCurMin>
where
    S: FixedName,
    FnCurMin: FnMut(&Effect<S>),
{
    fn default() -> Self {
        Self {
            _marker: Default::default(),
            once_current_to_min: Default::default(),
        }
    }
}

// just for test
pub fn create_empty_events<S: FixedName>() -> DynPropEventsImpl<S, fn(&Effect<S>)> {
    DynPropEventsImpl::default()
}

use std::{fmt::Debug, hash::Hash};

/// 可合并类型
pub trait Upsert {
    type Id: Eq + Hash + Clone + Debug;
    type IdRef<'a>: Eq + Hash + Copy + Clone + Debug
    where
        Self: 'a;

    /// 获取 id ，为能够自由组合字段，返回克隆后的所有权
    fn gen_id(&self) -> Self::Id;

    /// 为快速比较，避免多次获取 id 引起不必要的克隆
    fn id_ref<'a>(&'a self) -> Self::IdRef<'a>;
}

pub mod upsert_system {
    /// 直接替换，若想实现特殊效果（效果堆叠等）自己实现
    #[inline]
    pub fn replace<T: Sized>(old: &mut T, new: T) {
        *old = new;
    }
}

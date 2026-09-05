use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr::{
        effects::{EffId, EffIdRef, Effect},
        upserts::Upsert,
    },
};

/// 通过堆叠层数自动识别替换（堆叠）、卸载（削减）
pub struct BuffMeta<const SORTED: bool, S: FixedName> {
    /// 效果载体
    pub eff: Effect<S>,
    /// 堆叠层数，归 0 时触发卸载
    pub stack: u32,
    /// 堆叠上限，若不允许堆叠，则设置为 1
    pub max_stack: u32,
    /// 过期时减少的堆叠层数，不会触发下溢
    pub sub_stack_expired: u32,
}

impl<const SORTED: bool, S: FixedName> BuffMeta<SORTED, S> {
    pub fn add_stack(&mut self, other: &Self) {
        self.stack = self.max_stack.min(self.stack + other.stack)
    }

    pub(super) fn sub_stack(&mut self) {
        self.stack = self.stack.saturating_sub(self.sub_stack_expired);
    }

    pub(super) fn should_unmount(&self) -> bool {
        // 堆叠层数归零时触发卸载
        self.stack == 0
    }
}

impl<const SORTED: bool, S: FixedName> Upsert for BuffMeta<SORTED, S> {
    type Id = EffId<S>;
    type IdRef<'a>
        = EffIdRef<'a, S>
    where
        Self: 'a;

    fn gen_id(&self) -> Self::Id {
        self.eff.gen_id()
    }

    fn id_ref<'a>(&'a self) -> Self::IdRef<'a> {
        self.eff.id_ref()
    }
}

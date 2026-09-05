use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr::{
        effects::{EffId, EffIdRef, Effect},
        upserts::Upsert,
    },
};

/// buff 允许堆叠层数
pub struct BuffMeta<const SORTED: bool, S: FixedName> {
    /// 效果载体
    pub eff: Effect<S>,
    /// 堆叠层数，归 0 时触发卸载
    pub stack: u32,
    /// 堆叠上限，若不允许堆叠，则设置为 1
    pub max_stack: u32,
}

impl<const SORTED: bool, S: FixedName> BuffMeta<SORTED, S> {
    /// 与同类 buff 合并堆叠
    pub fn add_stack(&mut self, other: &Self) {
        self.stack = self.max_stack.min(self.stack + other.stack)
    }

    /// 削减堆叠层数，不会触发下溢
    pub fn sub_stack(&mut self, v: u32) {
        self.stack = self.stack.saturating_sub(v);
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

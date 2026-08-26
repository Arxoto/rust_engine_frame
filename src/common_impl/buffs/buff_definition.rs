use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr::{
        effects::{EffId, EffIdRef, Effect},
        upsert_container::Upsert,
    },
};

#[derive(Debug, Clone, Copy)]
pub enum BuffUpdateType {
    Reset,
    Unmount,
}

type BuffFn<This, Ctx> = Box<dyn Fn(&This, Ctx) -> Box<dyn Fn(&This, Ctx, BuffUpdateType)>>;

pub struct Buff<const SORTED: bool, S: FixedName, Ctx> {
    pub eff: Effect<S>,
    /// 堆叠层数，归 0 时触发卸载
    pub stack: u32,
    /// 堆叠上限，若不允许堆叠，则设置为 0
    pub max_stack: u32,
    /// 过期时减少的堆叠层数，不会触发下溢
    pub sub_stack_expired: u32,
    /// 挂载函数，返回方法闭包
    pub do_mount: BuffFn<Self, Ctx>,
}

impl<const SORTED: bool, S: FixedName, Ctx> Buff<SORTED, S, Ctx> {
    pub fn add_stack(&mut self, other: &Self) {
        self.stack = self.max_stack.min(self.stack + other.stack)
    }

    pub fn sub_stack(&mut self) {
        self.stack = self.stack.saturating_sub(self.sub_stack_expired);
    }

    pub fn gen_update_type(&self) -> BuffUpdateType {
        if self.stack == 0 {
            BuffUpdateType::Unmount
        } else {
            BuffUpdateType::Reset
        }
    }
}

impl<const SORTED: bool, S: FixedName, Ctx> Upsert for Buff<SORTED, S, Ctx> {
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

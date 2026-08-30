use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr::{
        effects::{EffId, EffIdRef, Effect},
        upsert_container::Upsert,
    },
};

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

pub trait BuffChanger<Ctx> {
    fn reset(&self, ctx: Ctx);
    fn unmount(&self, ctx: Ctx);
}

#[derive(Debug, Clone, Copy)]
pub enum BuffChangeType {
    Reset,
    Unmount,
}

impl BuffChangeType {
    fn apply<Ctx>(&self, buff_changer: &Box<dyn BuffChanger<Ctx>>, ctx: Ctx) {
        match self {
            BuffChangeType::Reset => buff_changer.reset(ctx),
            BuffChangeType::Unmount => buff_changer.unmount(ctx),
        }
    }
}

pub trait BuffLogic<const SORTED: bool, S: FixedName, Ctx> {
    fn mount_buff(&self, buff: &BuffMeta<SORTED, S>, ctx: Ctx) -> Box<dyn BuffChanger<Ctx>>;
}

impl<const SORTED: bool, S: FixedName> BuffMeta<SORTED, S> {
    pub fn add_stack(&mut self, other: &Self) {
        self.stack = self.max_stack.min(self.stack + other.stack)
    }

    pub fn sub_stack(&mut self) {
        self.stack = self.stack.saturating_sub(self.sub_stack_expired);
    }

    pub fn apply_buff_change<Ctx>(&self, buff_changer: &Box<dyn BuffChanger<Ctx>>, ctx: Ctx) {
        let change_type = self.gen_change_type();
        change_type.apply(buff_changer, ctx);
    }

    fn gen_change_type(&self) -> BuffChangeType {
        if self.stack == 0 {
            BuffChangeType::Unmount
        } else {
            BuffChangeType::Reset
        }
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

// region: impl for BuffChanger

/// 使用闭包语法糖实现了“匿名实现”的效果，以节约代码行数，
/// 但是闭包捕获的数据始终重复占据内存（因此若捕获数据很多，应该自己实现类型）
pub struct BuffChangerImpl<F1, F2> {
    reset_fn: F1,
    unmount_fn: F2,
}

impl<Ctx, F1, F2> BuffChanger<Ctx> for BuffChangerImpl<F1, F2>
where
    F1: Fn(Ctx),
    F2: Fn(Ctx),
{
    fn reset(&self, ctx: Ctx) {
        (self.reset_fn)(ctx)
    }
    fn unmount(&self, ctx: Ctx) {
        (self.unmount_fn)(ctx)
    }
}

impl<F1, F2> BuffChangerImpl<F1, F2> {
    pub fn new<Ctx>(reset_fn: F1, unmount_fn: F2) -> Box<dyn BuffChanger<Ctx>>
    where
        F1: Fn(Ctx) + 'static,
        F2: Fn(Ctx) + 'static,
    {
        Box::new(Self {
            reset_fn,
            unmount_fn,
        })
    }
}

// endregion

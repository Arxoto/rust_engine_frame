use crate::{
    base_lib::cores::{
        timers::{
            tick_timer::TickTimer,
            tick_trigger::{FewShotTickTrigger, InfiniteTickTrigger},
        },
        unify_types::FixedName,
    },
    common_impl::buffs::buff_definition::BuffMeta,
};

pub enum BuffTrigger{
    Inf(InfiniteTickTrigger),
    FewShot(FewShotTickTrigger),
}

pub enum BuffRuntime<const SORTED: bool, S: FixedName, Ctx> {
    Stat(TickTimer, Box<dyn BuffChanger<SORTED, S, Ctx>>),
    Shooter(BuffTrigger, Box<dyn BuffShooter<SORTED, S, Ctx>>),
}

// region: Shooter BuffShooter

/// 发生型 buff 的触发逻辑，自动为【闭包】实现了该特征
pub trait BuffShooter<const SORTED: bool, S: FixedName, Ctx> {
    fn shoot(&self, buff: &BuffMeta<SORTED, S>, ctx: Ctx);
}

impl<const SORTED: bool, S, Ctx, F> BuffShooter<SORTED, S, Ctx> for F
where
    S: FixedName,
    F: Fn(&BuffMeta<SORTED, S>, Ctx),
{
    fn shoot(&self, buff: &BuffMeta<SORTED, S>, ctx: Ctx) {
        self(buff, ctx)
    }
}

// endregion

// region: Stat BuffChanger

/// 状态 buff 的刷新逻辑
///
/// 可通过 [`BuffChangerFn::create`] 快速创建，但如果闭包捕获数据很多，建议自己实现
pub trait BuffChanger<const SORTED: bool, S: FixedName, Ctx> {
    fn reset(&self, buff: &BuffMeta<SORTED, S>, ctx: Ctx);
    fn unmount(&self, ctx: Ctx);
}

/// 状态 buff 的挂载逻辑
pub trait BuffMounter<const SORTED: bool, S: FixedName, Ctx> {
    fn mount_buff(
        &self,
        buff: &BuffMeta<SORTED, S>,
        ctx: Ctx,
    ) -> Box<dyn BuffChanger<SORTED, S, Ctx>>;
}

// region: impl for BuffChanger

/// 使用闭包语法糖实现了“匿名实现”的效果，以节约代码行数，
/// 但是闭包捕获的数据始终重复占据内存（因此若捕获数据很多，应该自己实现类型）
pub struct BuffChangerFn<F1, F2> {
    reset_fn: F1,
    unmount_fn: F2,
}

impl<const SORTED: bool, S: FixedName, Ctx, F1, F2> BuffChanger<SORTED, S, Ctx>
    for BuffChangerFn<F1, F2>
where
    F1: Fn(&BuffMeta<SORTED, S>, Ctx),
    F2: Fn(Ctx),
{
    fn reset(&self, buff: &BuffMeta<SORTED, S>, ctx: Ctx) {
        (self.reset_fn)(buff, ctx)
    }
    fn unmount(&self, ctx: Ctx) {
        (self.unmount_fn)(ctx)
    }
}

impl<F1, F2> BuffChangerFn<F1, F2> {
    pub fn create<const SORTED: bool, S: FixedName, Ctx>(
        reset_fn: F1,
        unmount_fn: F2,
    ) -> Box<dyn BuffChanger<SORTED, S, Ctx>>
    where
        F1: Fn(&BuffMeta<SORTED, S>, Ctx) + 'static,
        F2: Fn(Ctx) + 'static,
    {
        Box::new(Self {
            reset_fn,
            unmount_fn,
        })
    }
}

// endregion

// endregion

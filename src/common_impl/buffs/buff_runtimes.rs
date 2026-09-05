use crate::{
    base_lib::cores::{
        timers::tick_trigger::{FewShotTickTrigger, InfiniteTickTrigger},
        unify_types::FixedName,
    },
    common_impl::buffs::buff_definition::BuffMeta,
};

/// 计时触发器
pub enum BuffTrigger {
    Inf(InfiniteTickTrigger),
    FewShot(FewShotTickTrigger),
}

// region: 纯数据

pub struct BuffData<const SORTED: bool, S: FixedName> {
    pub trigger: BuffTrigger,
    pub meta: BuffMeta<SORTED, S>,
}

impl<const SORTED: bool, S: FixedName> BuffData<SORTED, S> {
    #[inline]
    pub fn new(trigger: BuffTrigger, meta: BuffMeta<SORTED, S>) -> Self {
        Self { trigger, meta }
    }
}

pub struct BuffDataMut<'a, const SORTED: bool, S: FixedName> {
    pub trigger: &'a mut BuffTrigger,
    pub meta: &'a mut BuffMeta<SORTED, S>,
}

impl<'a, const SORTED: bool, S: FixedName> BuffDataMut<'a, SORTED, S> {
    #[inline]
    pub fn new(trigger: &'a mut BuffTrigger, meta: &'a mut BuffMeta<SORTED, S>) -> Self {
        Self { trigger, meta }
    }
}

// endregion

/// buff 效果发射器
///
/// - 状态型 buff 应该只在合并时更新自身并发射效果
/// - 触发型 buff 应该在合并时跟新自身，在计时器触发时发射效果
pub enum BuffShooter<const SORTED: bool, S: FixedName> {
    /// 挂载 buff 触发合并
    MergedReset(BuffData<SORTED, S>),
    /// 计时器触发
    TimeTrigger,
}

/// buff 变更逻辑
///
/// 可通过 [`BuffChangerFn::create`] 快速创建，但如果闭包捕获数据很多，建议自己实现
pub trait BuffChanger<const SORTED: bool, S: FixedName, Ctx> {
    fn shoot(&mut self, origin: BuffDataMut<SORTED, S>, shooter: BuffShooter<SORTED, S>, ctx: Ctx);
    fn unmount(&mut self, buff_data: BuffData<SORTED, S>, ctx: Ctx);
}

pub type BuffRuntime<const SORTED: bool, S, Ctx> = Box<dyn BuffChanger<SORTED, S, Ctx>>;

/// buff 挂载逻辑
pub trait BuffMounter<const SORTED: bool, S: FixedName, Ctx> {
    fn mount_buff(&self, buff_data: &BuffData<SORTED, S>, ctx: Ctx) -> BuffRuntime<SORTED, S, Ctx>;
}

/// 使用闭包语法糖实现了“匿名实现”的效果，以节约代码行数，
/// 但是闭包捕获的数据始终重复占据内存（因此若捕获数据很多，应该自己实现类型）
pub struct BuffChangerFn<F1, F2> {
    shoot_fn: F1,
    unmount_fn: F2,
}

impl<const SORTED: bool, S: FixedName, Ctx, F1, F2> BuffChanger<SORTED, S, Ctx>
    for BuffChangerFn<F1, F2>
where
    F1: FnMut(BuffDataMut<SORTED, S>, BuffShooter<SORTED, S>, Ctx),
    F2: FnMut(BuffData<SORTED, S>, Ctx),
{
    fn shoot(&mut self, origin: BuffDataMut<SORTED, S>, shooter: BuffShooter<SORTED, S>, ctx: Ctx) {
        (self.shoot_fn)(origin, shooter, ctx)
    }
    fn unmount(&mut self, buff_data: BuffData<SORTED, S>, ctx: Ctx) {
        (self.unmount_fn)(buff_data, ctx)
    }
}

impl<F1, F2> BuffChangerFn<F1, F2> {
    pub fn create<const SORTED: bool, S: FixedName, Ctx>(
        shoot_fn: F1,
        unmount_fn: F2,
    ) -> BuffRuntime<SORTED, S, Ctx>
    where
        F1: FnMut(BuffDataMut<SORTED, S>, BuffShooter<SORTED, S>, Ctx) + 'static,
        F2: FnMut(BuffData<SORTED, S>, Ctx) + 'static,
    {
        Box::new(Self {
            shoot_fn,
            unmount_fn,
        })
    }
}

//! Buff 效果集合（包含计时器）
//!
//! 设计
//! - Buff 附带的效果由其生命周期方法进行管理（因此需要持有 SlotMap 的 Key 数据）
//! - Buff 需要每帧判断是否过期，每帧进行遍历，因此数据结构选择 DenseSlotMap
//! - Buff 允许堆叠，因此需要根据已知类型去搜索到已有的 Buff ，因此选择 FxHashMap 维护索引
//! - Timer 选择被 Buff 持有，防止二级索引
//!   - 思考是否可以拿 Timer 做 DenseSlotMap ，然后使用 Buff 做 SecondaryMap ，考虑到 Buff 数量（小于50个），一般 CPU 缓存 L1 完全够用
//!   - 每帧遍历检查过期时无需再次查询导致缓存未命中
//!   - 显示全量 Buff 时是否要遍历 Timer 并查询 Buff
//!   - 处理计时器过期时，若需要进行堆叠，则需要查询 Buff 导致缓存失效
//!   - 遍历时尽量使用 INF 而不是 None 以避免分支预测
//!
//! todo 合并逻辑 堆叠或替换 ； 过期逻辑 逆堆叠或移除

use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;

use crate::{
    base_lib::{
        cores::{
            timers::{
                tick_timer::TickTimer,
                tick_trigger::{FewShotTickTrigger, InfiniteTickTrigger},
            },
            unify_types::FixedName,
        },
        eff_attr::upserts::Upsert,
    },
    common_impl::buffs::buff_definition::{BuffChanger, BuffMeta, BuffTrigger},
};

type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;

#[inline]
fn new_fx_index_map<K, V>() -> FxIndexMap<K, V> {
    IndexMap::with_hasher(FxBuildHasher)
}

type BuffKey<const SORTED: bool, S> = <BuffMeta<SORTED, S> as Upsert>::Id;

pub enum BuffRuntime<const SORTED: bool, S: FixedName, Ctx> {
    Stat(TickTimer, Box<dyn BuffChanger<SORTED, S, Ctx>>),
    InfTriger(InfiniteTickTrigger, Box<dyn BuffTrigger<SORTED, S, Ctx>>),
    FewShotTriger(FewShotTickTrigger, Box<dyn BuffTrigger<SORTED, S, Ctx>>),
}

pub(super) struct BuffEntity<const SORTED: bool, S: FixedName, Ctx> {
    pub(super) meta: BuffMeta<SORTED, S>,
    pub(super) runtime: BuffRuntime<SORTED, S, Ctx>,
}

/// 在 OO 中，可以使用该实现，在 ECS 中建议一个 Buff 作为一个 subEntity
pub struct BuffCollection<const SORTED: bool, S: FixedName, Ctx> {
    ll: FxIndexMap<BuffKey<SORTED, S>, BuffEntity<SORTED, S, Ctx>>,
}

impl<const SORTED: bool, S: FixedName, Ctx> Default for BuffCollection<SORTED, S, Ctx> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SORTED: bool, S: FixedName, Ctx> BuffCollection<SORTED, S, Ctx> {
    pub fn new() -> Self {
        Self {
            ll: new_fx_index_map(),
        }
    }

    #[inline]
    pub(super) fn remove(
        &mut self,
        key: &BuffKey<SORTED, S>,
    ) -> Option<BuffEntity<SORTED, S, Ctx>> {
        if SORTED {
            self.ll.shift_remove(key)
        } else {
            self.ll.swap_remove(key)
        }
    }

    /// for upsert
    #[inline]
    pub(super) fn entry(
        &mut self,
        key: BuffKey<SORTED, S>,
    ) -> indexmap::map::Entry<'_, BuffKey<SORTED, S>, BuffEntity<SORTED, S, Ctx>> {
        self.ll.entry(key)
    }
}

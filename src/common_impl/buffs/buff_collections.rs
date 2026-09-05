//! Buff 效果集合（包含计时器）
//!
//! 设计
//! - Timer 选择被 Buff 持有，防止二级索引
//! - Buff 附带的效果由其生命周期方法进行管理（因此需要持有 SlotMap 的 Key 数据）
//! - Buff 需要每帧判断是否过期，每帧进行遍历，因此底层数据结构必须是 Vec ，可选 IndexMap/DenseSlotMap
//! - Buff 允许堆叠，因此需要根据已知类型去搜索到已有的 Buff ，因此选择 FxHashMap 维护索引
//! - 综上所述，选择 FxIndexMap 存放 buff

use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;

use crate::{
    base_lib::{cores::unify_types::FixedName, eff_attr::upserts::Upsert},
    common_impl::buffs::{
        buff_definition::BuffMeta,
        buff_runtimes::{BuffData, BuffDataMut, BuffRuntime, BuffShooter, BuffTrigger},
    },
};

type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;

#[inline]
fn new_fx_index_map<K, V>() -> FxIndexMap<K, V> {
    IndexMap::with_hasher(FxBuildHasher)
}

type BuffKey<const SORTED: bool, S> = <BuffMeta<SORTED, S> as Upsert>::Id;

/// buff 实体
pub(super) struct BuffEntity<const SORTED: bool, S: FixedName, Ctx> {
    trigger: BuffTrigger,
    pub(super) buff_meta: BuffMeta<SORTED, S>,
    runtime: BuffRuntime<SORTED, S, Ctx>,
}

impl<const SORTED: bool, S: FixedName, Ctx> BuffEntity<SORTED, S, Ctx> {
    #[inline]
    pub(super) fn new_from(
        buff_data: BuffData<SORTED, S>,
        runtime: BuffRuntime<SORTED, S, Ctx>,
    ) -> Self {
        Self {
            trigger: buff_data.trigger,
            buff_meta: buff_data.meta,
            runtime,
        }
    }

    pub(super) fn shoot(&mut self, shooter: BuffShooter<SORTED, S>, ctx: Ctx) {
        let Self {
            trigger,
            buff_meta,
            runtime,
        } = self;

        let buff_data = BuffDataMut::new(trigger, buff_meta);
        runtime.shoot(buff_data, shooter, ctx);
    }

    pub(super) fn unmount(self, ctx: Ctx) {
        let Self {
            trigger,
            buff_meta,
            mut runtime,
        } = self;

        let buff_data = BuffData::new(trigger, buff_meta);
        runtime.unmount(buff_data, ctx);
    }
}

/// buff 集合
///
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

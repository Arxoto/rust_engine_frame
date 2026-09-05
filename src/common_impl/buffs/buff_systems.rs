//! 堆叠逻辑交由 buff 层实现
//!
//! 堆叠维度
//! - [`BuffMeta`] 可以值覆盖、值选取最大值、层数堆叠、修改堆叠上限、修改堆叠削减
//! - [`TickTimer`] [`BuffTrigger`] 可以选择仅重置计时、也可以覆盖持续时间
//!
//! 复杂实现
//! 若要实现：在一定时间内逐渐变强，到时间后爆发，可通过父子buff实现
//!
//! 可与计时器组合实现复杂效果：
//! - 持续效果、持续触发效果
//! - 延迟生效效果，计时结束后自动添加另一个效果
//!
//! 对于如何堆叠，应该根据上层业务需求自己判断，不同堆叠优先级可以诞生有趣的策略，如：
//!
//! - 不同效果的延迟、频率、层数上限、层数、强度不同：初始效果决定频率和层数上限、中间快速堆叠层数、最后施加高强度、选择快速冷却的效果延续时长
//! - 某效果根据延迟生效的时长增加伤害，施加重置延迟效果，在最后造成大量伤害，这种机制也可替换成堆叠效果组合（同时施加重置延迟和堆叠层数两种效果）
//!
//! 注意：若允许不同来源的效果可叠加，那么必然会导致伤害结算存在误差：叠加产生的额外收益算谁的，这划分给谁都不合适，也许可以算成团队收益

use crate::{
    base_lib::{
        cores::{timers::tick_timer::TickTimer, unify_types::FixedName},
        eff_attr::upserts::Upsert,
    },
    common_impl::buffs::{
        buff_collections::{BuffCollection, BuffEntity},
        buff_definition::BuffMeta,
        buff_runtimes::{BuffMounter, BuffRuntime, BuffShooter, BuffTrigger},
    },
};

/// 合并的逻辑允许自定义
pub fn add_shooter_buff<const SORTED: bool, S, Ctx, Bst, Bmg>(
    buffs: &mut BuffCollection<SORTED, S, Ctx>,
    buff_meta: BuffMeta<SORTED, S>,
    buff_trigger: BuffTrigger,
    buff_shooter: Bst,
    buff_merge: &Bmg,
) where
    S: FixedName,
    Bst: BuffShooter<SORTED, S, Ctx> + 'static,
    Bmg: Fn(&mut BuffMeta<SORTED, S>, BuffMeta<SORTED, S>, &mut BuffTrigger, BuffTrigger),
{
    let key = buff_meta.gen_id();
    match buffs.entry(key) {
        indexmap::map::Entry::Occupied(mut entry) => {
            let buff_entity = entry.get_mut();
            if let BuffRuntime::Shooter(trigger, _shooter) = &mut buff_entity.runtime {
                buff_merge(&mut buff_entity.meta, buff_meta, trigger, buff_trigger);
            }
        }
        indexmap::map::Entry::Vacant(entry) => {
            let buff_entity = BuffEntity {
                meta: buff_meta,
                runtime: BuffRuntime::Shooter(buff_trigger, Box::new(buff_shooter)),
            };
            entry.insert(buff_entity);
        }
    }
}

/// 合并的逻辑允许自定义
pub fn add_stat_buff<const SORTED: bool, S, Ctx, Bmt, Bmg>(
    ctx: Ctx,
    buffs: &mut BuffCollection<SORTED, S, Ctx>,
    buff_meta: BuffMeta<SORTED, S>,
    buff_timer: TickTimer,
    buff_mount: &Bmt,
    buff_merge: &Bmg,
) where
    S: FixedName,
    Bmt: BuffMounter<SORTED, S, Ctx>,
    Bmg: Fn(&mut BuffMeta<SORTED, S>, BuffMeta<SORTED, S>, &mut TickTimer, TickTimer),
{
    let key = buff_meta.gen_id();
    match buffs.entry(key) {
        indexmap::map::Entry::Occupied(mut entry) => {
            let buff_entity = entry.get_mut();
            if let BuffRuntime::Stat(timer, buff_changer) = &mut buff_entity.runtime {
                buff_merge(&mut buff_entity.meta, buff_meta, timer, buff_timer);
                buff_changer.reset(&buff_entity.meta, ctx);
            }
        }
        indexmap::map::Entry::Vacant(entry) => {
            let buff_changer = buff_mount.mount_buff(&buff_meta, ctx);
            let buff_entity = BuffEntity {
                meta: buff_meta,
                runtime: BuffRuntime::Stat(buff_timer, buff_changer),
            };
            entry.insert(buff_entity);
        }
    }
}

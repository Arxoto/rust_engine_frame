//! 堆叠逻辑交由 buff 层实现
//!
//! 堆叠维度
//! - [`super::buff_definition::BuffMeta`] 可以值覆盖、值选取最大值、层数堆叠、修改堆叠上限、修改堆叠削减
//! - [`super::buff_runtimes::BuffTrigger`] 可以选择重置触发次数、覆盖触发上限，重置计时、覆盖持续时间
//!
//! 复杂实现举例
//! - 在一定时间内逐渐增强，到时间后触发技能
//!   - buff 在定时触发时给自身增加堆叠层数，而后更新效果
//!   - buff 卸载时移除效果，并发送事件触发技能
//! - 延迟触发
//!   - 祖先 buff 触发时将触发器替换，并开关启动子代 buff 触发逻辑
//!   - 合并 buff 时再次替换触发器，并关闭开关
//!   - buff 自身携带的触发器参数为子代的触发器，因此可以合并时修改触发次数和间隔
//!   - 祖先触发器存储在闭包里，每次合并时仅重置计时
//! - 延迟增强其他 buff
//!   - 卸载时推入其他 buff ，因为可变引用被持有，可以单独给 Ctx 增加一个 buffer 字段
//!
//! 对于如何堆叠，应该根据上层业务需求自己判断，不同堆叠优先级可以诞生有趣的策略，如：
//!
//! - 不同效果的延迟、频率、层数上限、层数、强度不同：初始效果决定频率和层数上限、中间快速堆叠层数、最后施加高强度、选择快速冷却的效果延续时长
//! - 某效果根据延迟生效的时长增加伤害，施加重置延迟效果，在最后造成大量伤害，这种机制也可替换成堆叠效果组合（同时施加重置延迟和堆叠层数两种效果）
//!
//! 注意：若允许不同来源的效果可叠加，那么必然会导致伤害结算存在误差：叠加产生的额外收益算谁的，这划分给谁都不合适，也许可以算成团队收益，然后均分

use crate::{
    base_lib::{
        cores::unify_types::FixedName,
        eff_attr::{effects::EffId, upserts::Upsert},
    },
    common_impl::buffs::{
        buff_collections::{BuffCollection, BuffEntity},
        buff_runtimes::{BuffData, BuffMounter, BuffShooter},
    },
};

pub fn add_buff<const SORTED: bool, S: FixedName, Ctx>(
    ctx: Ctx,
    buffs: &mut BuffCollection<SORTED, S, Ctx>,
    buff_data: BuffData<SORTED, S>,
    buff_mounter: &impl BuffMounter<SORTED, S, Ctx>,
) {
    let key = buff_data.meta.gen_id();
    match buffs.entry(key) {
        indexmap::map::Entry::Occupied(mut entry) => {
            let buff_entity = entry.get_mut();
            buff_entity.shoot(BuffShooter::MergedReset(buff_data), ctx);
        }
        indexmap::map::Entry::Vacant(entry) => {
            let buff_changer = buff_mounter.mount_buff(&buff_data, ctx);

            let buff_entity = BuffEntity::new_from(buff_data, buff_changer);
            entry.insert(buff_entity);
        }
    }
}

pub fn rm_buff<const SORTED: bool, S: FixedName, Ctx>(
    ctx: Ctx,
    buffs: &mut BuffCollection<SORTED, S, Ctx>,
    buff_id: EffId<S>,
) {
    if let Some(removed_buff) = buffs.remove(&buff_id) {
        removed_buff.unmount(ctx);
    }
}

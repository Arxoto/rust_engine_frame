//! 属性相关的 System 层
//!
//! ## 属性计算管线
//!
//! 首先刷新 [`super::stat_attrs`] [`super::bound_attrs`] 他们一般作为计算公式里的源端
//!
//! 而后刷新 [`super::bounded_attrs`] 应用计算公式得到结果
//!
//! 其中，有界属性的效果可分为两类：“无论如何都生效”和“根据结果决定是否生效”，他们的计算顺序应有区分
//!
//! - “无论如何都生效”的效果，应该先计算
//!   - 若具有层级，则使用复合属性 [`super::attr_layers`] ，先聚合同类效果再生效计算，以减少多层属性的计算次数
//!   - 每层计算完成后应该进行钳制，以确定下一层应生效多少效果值
//! - “根据结果决定是否生效”的效果，在后面计算，并且应该要求他们的顺序是确定的
//!   - 每次预判断结果时，都应结合上下限考虑，并且生效后应用钳制
//! - 最后提交有界属性的本次修改，作为下一帧快照值

use crate::base_lib::{
    cores::{
        timers::static_timer::{StaticTimeline, StaticTimer},
        unify_types::time_type,
    },
    eff_attr::{
        aggregators::InvalidModifier,
        bounded_attrs::BoundedAttr,
        modifier_collections::{ModifiableAttr, ModifierCollection},
    },
};

/// 每帧开头，提交有界属性的修改
#[inline]
pub fn do_commit_bounded_attr(attr: &mut BoundedAttr) {
    attr.commit_pending_value();
}

/// 每帧开头，令上一帧新加入的 pending 队列的修改器生效
///
/// 参考 `Bevy Changed<Stats>` 直接遍历连续内存，而不是维护一个脏队列（并发冲突）
#[inline]
pub fn commit_pending_modifiers<Modifier: InvalidModifier>(
    modifiers: &mut ModifierCollection<Modifier>,
) {
    modifiers.commit_pending();
}

/// 懒刷新，而不是每帧刷新
pub fn read_stat_attr_safety<Modifier: InvalidModifier, Attr: ModifiableAttr<Modifier>>(
    attr: &mut Attr,
    modifiers: &mut ModifierCollection<Modifier>,
) -> f64 {
    if modifiers.is_dirty() {
        modifiers.reset_flag();

        attr.refresh_value(modifiers.iter());
    }
    attr.get_current()
}

/// 重置时间线（使用 f64 或 Duration 作为时间类型，基本无需重置时间线）
pub fn try_reset_timeline<'a>(
    timeline: &mut StaticTimeline,
    timers_iter: impl Iterator<Item = &'a mut StaticTimer>,
) {
    let should_reset_timeline = timeline.current_time() >= time_type::RESET_TIMELINE_PERIOD;
    if should_reset_timeline {
        let diff = timeline.reset_timeline_and_get_diff();
        for ele in timers_iter {
            ele.fix_timeline_diff(diff);
        }
    }
}

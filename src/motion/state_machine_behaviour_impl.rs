//! 行为系统的状态机实现

use crate::{
    cores::unify_type::FixedString,
    motion::{
        movement_impl::MovementMode,
        state_machine_types_impl::{EnterParam, FrameParam, MovementBehaviour, PhyParam},
    },
};

/// 行为状态该机
///
/// 每个行为有自己的进入条件，每帧遍历检查状态切换，状态与状态之间解耦合
///
/// 渲染帧的效果类型暂且保留在泛型中，感觉可能会复杂化，后续可简化为动画名称
#[derive(Default)]
pub struct BehaviourMachine<S, FrameEff, PhyEff>
where
    S: FixedString,
{
    pub(crate) stats: Vec<Box<dyn MovementBehaviour<S, FrameEff, PhyEff>>>,
    pub(crate) current_id: usize,
}

impl<S: FixedString, FrameEff, PhyEff> BehaviourMachine<S, FrameEff, PhyEff> {
    fn fetch_next_stat_id(&self, p: &EnterParam) -> Option<usize> {
        for (id, ele) in self.stats.iter().enumerate() {
            if ele.will_enter(p) && id != self.current_id {
                return Some(id);
            }
        }
        return None;
    }

    /// 更新状态 返回运动模式的切换
    pub(crate) fn update_stat(
        &mut self,
        p: &EnterParam,
    ) -> (Option<MovementMode>, Option<MovementMode>) {
        let Some(next_stat_id) = self.fetch_next_stat_id(p) else {
            return (None, None);
        };

        let mut old_movement_mode = None;
        if let Some(stat) = self.stats.get_mut(self.current_id) {
            stat.on_exit();
            old_movement_mode = Some(stat.get_movement_mode());
        }

        self.current_id = next_stat_id;

        let mut new_movement_mode = None;
        if let Some(stat) = self.stats.get_mut(self.current_id) {
            stat.on_enter();
            new_movement_mode = Some(stat.get_movement_mode());
        }

        (old_movement_mode, new_movement_mode)
    }

    /// 渲染帧执行
    pub(crate) fn tick_frame(&mut self, p: &FrameParam<S>) -> Option<FrameEff> {
        let Some(stat) = self.stats.get_mut(self.current_id) else {
            return None;
        };
        Some(stat.tick_frame(p))
    }

    /// 物理帧执行
    pub(crate) fn tick_physics(&mut self, p: &PhyParam<S>) -> Option<PhyEff> {
        let Some(stat) = self.stats.get_mut(self.current_id) else {
            return None;
        };
        Some(stat.tick_physics(p))
    }

    /// 初始化时新增
    pub fn add_behaviour(&mut self, b: Box<dyn MovementBehaviour<S, FrameEff, PhyEff>>) {
        self.stats.push(b);
    }
}

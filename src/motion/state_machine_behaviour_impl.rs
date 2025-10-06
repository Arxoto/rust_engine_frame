//! 行为系统的状态机实现

use crate::{
    cores::unify_type::FixedString,
    motion::{
        movement_impl::MovementMode,
        state_machine_types_impl::{FrameParam, MovementBehaviour, PhyParam},
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
    fn fetch_next_stat_id(&self, enter_param: &PhyParam<S>) -> Option<usize> {
        for (id, ele) in self.stats.iter().enumerate() {
            if ele.will_enter(enter_param) && id != self.current_id {
                return Some(id);
            }
        }
        return None;
    }

    /// 更新状态 返回运动模式的切换
    pub(crate) fn update_stat(
        &mut self,
        enter_param: &PhyParam<S>,
    ) -> (Option<MovementMode>, Option<MovementMode>) {
        let Some(next_stat_id) = self.fetch_next_stat_id(enter_param) else {
            // do not update_stat
            let current_movement_mode = self
                .stats
                .get_mut(self.current_id)
                .map(|s| s.get_movement_mode());
            // return new_stat = None
            return (current_movement_mode, None);
        };

        // do update

        let mut old_movement_mode = None;
        if let Some(stat) = self.stats.get_mut(self.current_id) {
            stat.on_exit();
            old_movement_mode = Some(stat.get_movement_mode());
        }

        self.current_id = next_stat_id;

        let mut new_movement_mode = None; // never
        if let Some(stat) = self.stats.get_mut(self.current_id) {
            stat.on_enter();
            new_movement_mode = Some(stat.get_movement_mode());
        }

        (old_movement_mode, new_movement_mode)
    }

    /// 渲染帧执行 返回渲染效果
    ///
    /// 行为侧重逻辑，返回值一般认为是临时生成的，所以返回所有权
    pub(crate) fn process_frame(&mut self, frame_param: &FrameParam<S>) -> Option<FrameEff> {
        if let Some(stat) = self.stats.get_mut(self.current_id) {
            Some(stat.tick_frame(frame_param))
        } else {
            None
        }
    }

    /// 物理帧执行 返回物理效果
    ///
    /// 行为侧重逻辑，返回值一般认为是临时生成的，所以返回所有权
    ///
    /// P.S. 若想要叠加状态转换后的效果 可将帧处理和效果获取拆开 支持两者的自定义顺序
    /// process 方法侧重处理 get 方法侧重获取效果
    pub(crate) fn tick_physics(&mut self, phy_param: &PhyParam<S>) -> Option<PhyEff> {
        if let Some(stat) = self.stats.get_mut(self.current_id) {
            Some(stat.tick_physics(phy_param))
        } else {
            None
        }
    }

    /// 初始化时新增
    pub fn add_behaviour(&mut self, b: Box<dyn MovementBehaviour<S, FrameEff, PhyEff>>) {
        self.stats.push(b);
    }
}

//! 行为系统的状态机实现

use crate::{
    cores::unify_type::FixedString,
    motions::{
        state_machine_frame_param::FrameParam, state_machine_phy_eff::MotionData,
        state_machine_phy_param::PhyParam, state_machine_types::MotionBehaviour,
    },
};

/// 行为状态该机
///
/// 每个行为有自己的进入条件，每帧遍历检查状态切换，状态与状态之间解耦合
///
/// 渲染帧的效果类型暂且保留在泛型中，感觉可能会复杂化，后续可简化为动画名称
pub struct BehaviourMachine<S, FrameEff, PhyEff>
where
    S: FixedString,
{
    pub(crate) stats: Vec<Box<dyn MotionBehaviour<S, FrameEff, PhyEff>>>,
    pub(crate) current_id: usize,
    motion_data: MotionData,
}

impl<S: FixedString, FrameEff, PhyEff> BehaviourMachine<S, FrameEff, PhyEff> {
    pub fn new(data: MotionData) -> Self {
        Self {
            stats: Vec::new(),
            current_id: usize::MAX, // 第一次状态转换使旧状态为 None
            motion_data: data,
        }
    }

    /// 设置行为数据集
    pub fn set_motion_data(&mut self, data: MotionData) {
        self.motion_data = data;
    }

    /// 返回的值必定有效
    fn fetch_next_stat_id(&self, enter_param: &PhyParam<S>) -> Option<usize> {
        for (id, ele) in self.stats.iter().enumerate() {
            if ele.will_enter(enter_param) && id != self.current_id {
                return Some(id);
            }
        }
        None
    }

    /// 更新状态 返回运动模式的切换
    pub(crate) fn update_stat(&mut self, enter_param: &PhyParam<S>) {
        let Some(next_stat_id) = self.fetch_next_stat_id(enter_param) else {
            // do not update_stat
            return;
        };

        // do update

        if let Some(stat) = self.stats.get_mut(self.current_id) {
            stat.on_exit(enter_param);
        }

        self.current_id = next_stat_id;

        if let Some(stat) = self.stats.get_mut(self.current_id) {
            stat.on_enter(enter_param);
        }
    }

    /// 渲染帧执行 返回渲染效果
    pub(crate) fn tick_frame(&mut self, frame_param: &FrameParam<S>) -> Option<FrameEff> {
        let stat = self.stats.get_mut(self.current_id)?;
        Some(stat.tick_frame(frame_param))
    }

    /// 物理帧执行 返回物理效果
    ///
    /// 行为侧重逻辑处理，因此命名有所区别
    pub(crate) fn process_physics(&mut self, phy_param: &mut PhyParam<S>) -> Option<PhyEff> {
        let stat = self.stats.get_mut(self.current_id)?;
        Some(stat.process_physics(&mut (phy_param, &self.motion_data)))
    }

    /// 合并帧处理和状态更新
    pub(crate) fn process_and_update(&mut self, phy_param: &mut PhyParam<S>) -> Option<PhyEff> {
        let phy_eff = self.process_physics(phy_param);
        self.update_stat(phy_param);
        phy_eff
    }

    /// 初始化时新增
    pub fn add_behaviour(&mut self, b: Box<dyn MotionBehaviour<S, FrameEff, PhyEff>>) {
        self.stats.push(b);
    }
}

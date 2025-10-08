//! 玩家角色状态机实现
//!
//! 代码检视：
//! - 注意内部维护的字段状态
//!
//! 不实现单元测试，因为行为系统不方便实现单元测试，建议一同进行集成测试
//!
//! 测试点：对 phy_param 的修改是否正确生效
//! - action_duration 在动作切换后是否正确清零
//! - motion_changed 在行为切换后是否能触发动作系统的对应动作
//! - 预输入指令的回响是否被外部系统正确收集（ [`PlayerController`] 是否接收到回响，不重复指令下发）

use crate::{
    cores::unify_type::FixedString,
    motions::{
        state_machine_action::ActionMachine,
        state_machine_behaviour::BehaviourMachine,
        state_machine_frame_eff::FrameEff,
        state_machine_param::{FrameParam, PhyParam},
        state_machine_phy_eff::PhyEff,
        state_machine_types::EffGenerator,
    },
};

/// 玩家角色状态机
///
/// 组合了行为与动作系统
pub struct PlayerMachine<S>
where
    S: FixedString,
{
    pub(crate) action_machine: ActionMachine<S, PhyEff>,
    pub(crate) behaviour_machine: BehaviourMachine<S, FrameEff<S>, PhyEff>,

    // inner field
    /// 动作持续时间
    action_duration: f64,
}

impl<S> PlayerMachine<S>
where
    S: FixedString,
{
    pub fn new(
        action_machine: ActionMachine<S, PhyEff>,
        behaviour_machine: BehaviourMachine<S, FrameEff<S>, PhyEff>,
    ) -> Self {
        Self {
            action_machine,
            behaviour_machine,
            action_duration: 0.0,
        }
    }

    /// 渲染帧执行
    pub fn tick_frame<FE, PE, EG: EffGenerator<S, FE, PE>>(&mut self, p: &FrameParam<S>) -> FE {
        let frame_eff = self.behaviour_machine.tick_frame(p);
        let anim_name = self.action_machine.tick_frame(p);

        EG::gen_frame_eff(anim_name, frame_eff)
    }

    /// 物理帧执行
    ///
    /// 先进行帧处理，后进行状态更新，保证逻辑自洽（帧处理是基于帧开始的状态进行的）
    pub fn process_physics<FE, PE, EG: EffGenerator<S, FE, PE>>(
        &mut self,
        phy_param: &mut PhyParam<S>,
    ) -> PE {
        // 内部维护状态的参数
        // 考虑到一致性 应仅对参数赋初始值 而不做修改，但实际上随着动作的改变修改了 action_duration

        // porcess self
        // ===========================
        // 动作持续时间
        self.action_duration += phy_param.delta;
        // ===========================
        // fix param
        phy_param.action_duration = Some(self.action_duration);

        // process machine
        // ===========================
        // for behaviour_machine
        let phy_param_b = &mut phy_param.clone();
        let (phy_eff_b, motion_changed) = self.behaviour_machine.process_and_update(phy_param_b);
        // updated motion_changed
        phy_param.motion_changed = Some(motion_changed);

        // for action_machine
        let (phy_eff_a, action_updated) = self.action_machine.tick_and_update(phy_param);
        // updated action_duration
        if action_updated {
            self.action_duration = 0.0;
        }

        // do echo for behaviour_machine
        // no need echo for action_machine
        phy_param.op_echo_with(phy_param_b);

        EG::gen_phy_eff(phy_eff_a, phy_eff_b)
    }
}

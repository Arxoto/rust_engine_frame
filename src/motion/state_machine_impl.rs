//! 玩家角色状态机实现

use crate::{
    cores::unify_type::FixedString,
    motion::{
        movement_action_impl::MovementActionEvent,
        state_machine_action_impl::ActionMachine,
        state_machine_behaviour_impl::BehaviourMachine,
        state_machine_types_impl::{FrameParam, PhyParam},
    },
};

/// 玩家角色状态机
///
/// 组合了行为与动作系统
#[derive(Default)]
pub struct PlayerMachine<S, FrameEff, PhyEff>
where
    S: FixedString,
    FrameEff: TryFrom<S>,
{
    pub(crate) action_machine: ActionMachine<S, PhyEff>,
    pub(crate) behaviour_machine: BehaviourMachine<S, FrameEff, PhyEff>,

    // inner stat
    /// 动作的持续时间
    action_duration: f64,
}

impl<S, FrameEff, PhyEff> PlayerMachine<S, FrameEff, PhyEff>
where
    S: FixedString,
    FrameEff: TryFrom<S>,
    PhyEff: Clone,
{
    /// 事件触发
    pub fn trigger_by_event(&mut self, e: &MovementActionEvent) {
        self.action_machine.update_action_by_event(e);
    }

    /// 渲染帧执行
    /// 
    /// 先进行帧处理，后进行状态更新，保证逻辑自洽（帧处理是基于帧开始的状态进行的） todo
    pub fn tick_frame(&mut self, p: &mut FrameParam<S>) -> Option<FrameEff> {
        // 内部维护状态的参数
        // 考虑到一致性 应仅对参数赋初始值 而不做修改，但实际上随着动作的改变修改了 action_duration

        // action_duration 动作持续时间
        self.action_duration += p.delta;
        p.action_duration = self.action_duration;

        // update behaviour_machine
        let movement_changed = self.behaviour_machine.update_stat(p);
        // set exit_param with movement_changed
        p.movement_changed = movement_changed;

        // update action_machine
        let action_updated = self.action_machine.update_action_by_tick(p);
        // 动作更新 需要同步更新
        if action_updated {
            self.action_duration = 0.0;
            p.action_duration = self.action_duration;
        }

        // tick
        let frame_eff = self.behaviour_machine.tick_frame(p);
        let anim_name = self.action_machine.tick_frame(p);

        // anim_name first, and then frame_eff
        match FrameEff::try_from(anim_name.clone()) {
            Ok(frame_eff) => Some(frame_eff),
            Err(_) => frame_eff,
        }
    }

    /// 物理帧执行
    pub fn tick_physics(&mut self, p: &PhyParam<S>) -> Option<PhyEff> {
        let phy_eff_b = self.behaviour_machine.tick_physics(p);
        let phy_eff_a = self.action_machine.tick_physics(p);

        phy_eff_a.map(|p| p.clone()).or(phy_eff_b)
    }
}

//! 玩家角色状态机实现

use crate::{
    cores::unify_type::FixedString,
    motion::{
        movement_action_impl::MovementActionEvent,
        state_machine_action_impl::ActionMachine,
        state_machine_behaviour_impl::BehaviourMachine,
        state_machine_types_impl::{EffGenerator, FrameEff, FrameParam, PhyEff, PhyParam},
    },
};

/// 玩家角色状态机
///
/// 组合了行为与动作系统
#[derive(Default)]
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
    /// 事件触发
    pub fn trigger_by_event(&mut self, e: &MovementActionEvent) {
        let action_updated = self.action_machine.update_action_by_event(e);
        if action_updated {
            self.action_duration = 0.0;
        }
    }

    /// 渲染帧执行
    ///
    /// 先进行帧处理，后进行状态更新，保证逻辑自洽（帧处理是基于帧开始的状态进行的）
    pub fn tick_frame<FE, PE, EG: EffGenerator<S, FE, PE>>(&mut self, p: &FrameParam<S>) -> FE {
        // 内部维护状态的参数
        // 考虑到一致性 应仅对参数赋初始值 而不做修改，但实际上随着动作的改变修改了 action_duration

        // porcess self
        // ===========================
        // 动作持续时间
        self.action_duration += p.delta;
        // ===========================
        // generate new param
        let frame_param = &mut FrameParam {
            action_duration: Some(self.action_duration),
            ..p.clone()
        };

        // porcess machine
        self.behaviour_machine.process_frame(frame_param);
        self.action_machine.process_frame(frame_param);

        // update behaviour_machine
        let movement_changed = self.behaviour_machine.update_stat(frame_param);
        // set exit_param with movement_changed
        match movement_changed {
            (Some(the_old), Some(the_new)) => {
                frame_param.movement_changed = Some((the_old, the_new))
            }
            _ => {}
        }

        // update action_machine
        let action_updated = self.action_machine.update_action_by_tick(frame_param);
        // 动作更新 需要同步更新
        if action_updated {
            self.action_duration = 0.0;
        }

        // get eff
        let frame_eff = self.behaviour_machine.get_frame_eff();
        let anim_name = self.action_machine.get_frame_eff();

        EG::gen_frame_eff(anim_name, frame_eff)
    }

    /// 物理帧执行
    pub fn tick_physics<FE, PE, EG: EffGenerator<S, FE, PE>>(&mut self, p: &PhyParam<S>) -> PE {
        let phy_eff_b = self.behaviour_machine.tick_physics(p);
        let phy_eff_a = self.action_machine.tick_physics(p);

        EG::gen_phy_eff(phy_eff_a, phy_eff_b)
    }
}

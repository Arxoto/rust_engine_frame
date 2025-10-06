//! 玩家角色状态机实现

use crate::{
    cores::unify_type::FixedString,
    motion::{
        movement_action_impl::MovementActionEvent,
        movement_impl::MovementMode,
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
    /// 渲染帧执行
    pub fn tick_frame<FE, PE, EG: EffGenerator<S, FE, PE>>(&mut self, p: &FrameParam<S>) -> FE {
        let frame_eff = self.behaviour_machine.process_frame(p);
        let anim_name = self.action_machine.tick_frame(p);

        EG::gen_frame_eff(anim_name, frame_eff)
    }

    fn try_update_action(
        &mut self,
        phy_param: &PhyParam<S>,
        current_movement: Option<MovementMode>,
    ) -> bool {
        // 将事件更新与帧更新放在一处容易维护
        let updated = match current_movement {
            Some(movement) => self.try_update_action_event(phy_param, movement),
            _ => false,
        };
        // 不允许状态连续更新
        updated || self.action_machine.update_action_by_tick(phy_param)
    }

    fn try_update_action_event(
        &mut self,
        phy_param: &PhyParam<S>,
        movement: MovementMode,
    ) -> bool {
        for event in phy_param.to_instructions() {
            let updated = self
                .action_machine
                .update_action_by_event(&MovementActionEvent::new(event, movement));
            if updated {
                return true;
            }
        }
        false
    }

    /// 物理帧执行
    ///
    /// 先进行帧处理，后进行状态更新，保证逻辑自洽（帧处理是基于帧开始的状态进行的）
    pub fn tick_physics<FE, PE, EG: EffGenerator<S, FE, PE>>(&mut self, p: &PhyParam<S>) -> PE {
        // 内部维护状态的参数
        // 考虑到一致性 应仅对参数赋初始值 而不做修改，但实际上随着动作的改变修改了 action_duration

        // porcess self
        // ===========================
        // 动作持续时间
        self.action_duration += p.delta;
        // ===========================
        // generate new param
        let phy_param = &mut PhyParam {
            action_duration: Some(self.action_duration),
            ..p.clone()
        };

        // porcess machine
        let phy_eff_b = self.behaviour_machine.tick_physics(p);
        let phy_eff_a = self.action_machine.tick_physics(p).cloned();

        // update behaviour_machine
        let movement_changed = self.behaviour_machine.update_stat(phy_param);
        // set exit_param with movement_changed
        match movement_changed {
            (Some(the_old), Some(the_new)) => {
                phy_param.movement_changed = Some((the_old, the_new))
            }
            _ => {}
        }

        // update action_machine
        let action_updated = self.try_update_action(phy_param, movement_changed.0);
        // 动作更新 需要同步更新
        if action_updated {
            self.action_duration = 0.0;
        }

        EG::gen_phy_eff(phy_eff_a, phy_eff_b)
    }
}

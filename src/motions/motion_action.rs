//! 运动模式对【事件】和【退出逻辑】的功能增强

use crate::{
    cores::unify_type::FixedString,
    motions::{
        abstracts::action_types::{ActionEvent, ActionExitLogic},
        abstracts::player_input::PlayerOperation,
        action_impl::{ActionBaseEvent, ActionBaseExitLogic},
        motion_mode::MotionMode,
        state_machine_param::PhyParam,
    },
};

/// 和 [`MotionMode`] 组合来实现 [`ActionEvent`]
///
/// 支持复杂的触发条件判断
///
/// 一般基于事件或信号机制去响应
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct MotionActionEvent {
    pub(crate) event: ActionBaseEvent,
    pub(crate) motion: MotionMode,
}

impl ActionEvent for MotionActionEvent {}

impl MotionActionEvent {
    pub fn new(event: ActionBaseEvent, motion: MotionMode) -> Self {
        Self { event, motion }
    }

    /// 当事件对运动模式没有要求时，全量生成
    pub fn new_each_motion(event: ActionBaseEvent) -> Vec<Self> {
        MotionMode::each_mode()
            .into_iter()
            .map(|motion| Self::new(event, *motion))
            .collect()
    }
}

/// 和 [`MotionMode`] 互斥来实现 [`ActionExitLogic`]
///
/// 支持 运动状态切换 导致的 动作切换
///
/// 一般用于每帧检查
#[derive(Clone, Debug)]
pub enum MotionActionExitLogic<S: FixedString> {
    ExitLogic(ActionBaseExitLogic<S>),
    MotionChange(MotionMode, MotionMode),
}

impl<S: FixedString> MotionActionExitLogic<S> {
    fn should_exit_by_logic(exit_logic: &ActionBaseExitLogic<S>, param: &PhyParam<S>) -> bool {
        match exit_logic {
            ActionBaseExitLogic::AnimFinished(anim_name) => {
                param.anim_finished && param.anim_name == *anim_name
            }
            ActionBaseExitLogic::MoveAfter(the_time) => {
                param.move_direction.op_active()
                    && param
                        .action_duration
                        .map(|duration_time| duration_time > *the_time)
                        .unwrap_or(false)
            }
            ActionBaseExitLogic::JumpAfter(the_time) => {
                param.jump_once.op_active()
                    && param
                        .action_duration
                        .map(|duration_time| duration_time > *the_time)
                        .unwrap_or(false)
            }
            ActionBaseExitLogic::AttackWhen(anim_name) => {
                param.attack_once.op_active() && param.anim_name == *anim_name
            }
        }
    }

    fn should_exit_by_motion_change(
        old_motion: &MotionMode,
        new_motion: &MotionMode,
        param: &PhyParam<S>,
    ) -> bool {
        match param.motion_changed {
            Some((Some(the_old), Some(the_new))) => {
                the_old == *old_motion && the_new == *new_motion
            }
            _ => false,
        }
    }
}

impl<S: FixedString> ActionExitLogic<PhyParam<S>> for MotionActionExitLogic<S> {
    fn should_exit(&self, exit_param: &PhyParam<S>) -> bool {
        match self {
            MotionActionExitLogic::ExitLogic(exit_logic) => {
                Self::should_exit_by_logic(exit_logic, exit_param)
            }
            MotionActionExitLogic::MotionChange(old_motion, new_motion) => {
                Self::should_exit_by_motion_change(old_motion, new_motion, exit_param)
            }
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::motions::abstracts::player_pre_input::PreInputInstruction;

    use super::*;

    #[test]
    fn exit_logic_anim_finished() {
        let exit_logic = MotionActionExitLogic::ExitLogic(ActionBaseExitLogic::AnimFinished(""));

        let param: PhyParam<&'static str> = PhyParam {
            anim_finished: true,
            anim_name: "",
            ..Default::default()
        };
        assert!(exit_logic.should_exit(&param));

        let param: PhyParam<&'static str> = PhyParam {
            anim_finished: false,
            anim_name: "",
            ..Default::default()
        };
        assert!(!exit_logic.should_exit(&param));

        let param: PhyParam<&'static str> = PhyParam {
            anim_finished: true,
            anim_name: " ",
            ..Default::default()
        };
        assert!(!exit_logic.should_exit(&param));
    }

    #[test]
    fn exit_logic_move_after() {
        let exit_logic = MotionActionExitLogic::ExitLogic(ActionBaseExitLogic::MoveAfter(1.2));

        let param: PhyParam<String> = PhyParam {
            action_duration: Some(1.0),
            move_direction: 0.0.into(),
            ..Default::default()
        };
        assert!(!exit_logic.should_exit(&param));

        let param: PhyParam<String> = PhyParam {
            action_duration: Some(1.0),
            move_direction: 1.0.into(),
            ..Default::default()
        };
        assert!(!exit_logic.should_exit(&param));

        let param: PhyParam<String> = PhyParam {
            action_duration: Some(1.2),
            move_direction: 0.0.into(),
            ..Default::default()
        };
        assert!(!exit_logic.should_exit(&param));

        let param: PhyParam<String> = PhyParam {
            action_duration: Some(1.2),
            move_direction: 1.0.into(),
            ..Default::default()
        };
        assert!(!exit_logic.should_exit(&param));

        let param: PhyParam<String> = PhyParam {
            action_duration: Some(1.3),
            move_direction: 0.0.into(),
            ..Default::default()
        };
        assert!(!exit_logic.should_exit(&param));

        let param: PhyParam<String> = PhyParam {
            action_duration: Some(1.3),
            move_direction: 1.0.into(),
            ..Default::default()
        };
        assert!(exit_logic.should_exit(&param));
    }

    #[test]
    fn exit_logic_jump_after() {
        let exit_logic = MotionActionExitLogic::ExitLogic(ActionBaseExitLogic::JumpAfter(1.2));

        let param: PhyParam<String> = PhyParam {
            action_duration: Some(1.3),
            jump_once: PreInputInstruction(false, Default::default()),
            ..Default::default()
        };
        assert!(!exit_logic.should_exit(&param));

        let param: PhyParam<String> = PhyParam {
            action_duration: Some(1.3),
            jump_once: PreInputInstruction(true, Default::default()),
            ..Default::default()
        };
        assert!(exit_logic.should_exit(&param));
    }
}

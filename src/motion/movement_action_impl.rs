//! 运动模式对【事件】和【退出逻辑】的功能增强

use crate::{
    cores::unify_type::FixedString,
    motion::{
        action_impl::{ActionBaseEvent, ActionBaseExitLogic},
        action_types::{ActionEvent, ActionExitLogic},
        movement_impl::MovementMode,
        player_operation::PlayerOperation,
        state_machine_types_impl::FrameParam,
    },
};

/// 和 [`MovementMode`] 组合来实现 [`ActionEvent`]
///
/// 支持复杂的触发条件判断
///
/// 一般基于事件或信号机制去响应
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct MovementActionEvent {
    pub(crate) event: ActionBaseEvent,
    pub(crate) movement: MovementMode,
}

impl ActionEvent for MovementActionEvent {}

impl MovementActionEvent {
    pub fn new(event: ActionBaseEvent, movement: MovementMode) -> Self {
        Self { event, movement }
    }

    /// 当事件对运动模式没有要求时，全量生成
    pub fn new_each_movement(event: ActionBaseEvent) -> Vec<Self> {
        MovementMode::each_mode()
            .into_iter()
            .map(|movement| Self::new(event, *movement))
            .collect()
    }
}

/// 和 [`MovementMode`] 互斥来实现 [`ActionExitLogic`]
///
/// 支持 运动状态切换 导致的 动作切换
///
/// 一般用于每帧检查
#[derive(Clone, Debug)]
pub enum MovementActionExitLogic<S: FixedString> {
    ExitLogic(ActionBaseExitLogic<S>),
    MovementChange(MovementMode, MovementMode),
}

impl<S: FixedString> MovementActionExitLogic<S> {
    fn should_exit_by_logic(exit_logic: &ActionBaseExitLogic<S>, param: &FrameParam<S>) -> bool {
        match exit_logic {
            ActionBaseExitLogic::AnimFinished(anim_name) => {
                param.anim_finished && param.anim_name == *anim_name
            }
            ActionBaseExitLogic::MoveAfter(the_time) => {
                param.want_move.operation_active()
                    && param
                        .action_duration
                        .map(|duration_time| duration_time > *the_time)
                        .unwrap_or(false)
            }
            ActionBaseExitLogic::JumpAfter(the_time) => {
                param.want_jump
                    && param
                        .action_duration
                        .map(|duration_time| duration_time > *the_time)
                        .unwrap_or(false)
            }
            ActionBaseExitLogic::AttackWhen(anim_name) => {
                param.want_attack && param.anim_name == *anim_name
            }
        }
    }

    fn should_exit_by_movement_change(
        old_movement: &MovementMode,
        new_movement: &MovementMode,
        param: &FrameParam<S>,
    ) -> bool {
        match param.movement_changed {
            Some((the_old, the_new)) => the_old == *old_movement && the_new == *new_movement,
            None => false,
        }
    }
}

impl<S: FixedString> ActionExitLogic<FrameParam<S>> for MovementActionExitLogic<S> {
    fn should_exit(&self, exit_param: &FrameParam<S>) -> bool {
        match self {
            MovementActionExitLogic::ExitLogic(exit_logic) => {
                Self::should_exit_by_logic(exit_logic, exit_param)
            }
            MovementActionExitLogic::MovementChange(old_movement, new_movement) => {
                Self::should_exit_by_movement_change(old_movement, new_movement, exit_param)
            }
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn exit_logic_anim_finished() {
        let exit_logic = MovementActionExitLogic::ExitLogic(ActionBaseExitLogic::AnimFinished(""));

        let param: FrameParam<&'static str> = FrameParam {
            anim_finished: true,
            anim_name: "",
            ..Default::default()
        };
        assert!(exit_logic.should_exit(&param));

        let param: FrameParam<&'static str> = FrameParam {
            anim_finished: false,
            anim_name: "",
            ..Default::default()
        };
        assert!(!exit_logic.should_exit(&param));

        let param: FrameParam<&'static str> = FrameParam {
            anim_finished: true,
            anim_name: " ",
            ..Default::default()
        };
        assert!(!exit_logic.should_exit(&param));
    }

    #[test]
    fn exit_logic_move_after() {
        let exit_logic = MovementActionExitLogic::ExitLogic(ActionBaseExitLogic::MoveAfter(1.2));

        let param: FrameParam<String> = FrameParam {
            action_duration: Some(1.0),
            want_move: 0.0,
            ..Default::default()
        };
        assert!(!exit_logic.should_exit(&param));

        let param: FrameParam<String> = FrameParam {
            action_duration: Some(1.0),
            want_move: 1.0,
            ..Default::default()
        };
        assert!(!exit_logic.should_exit(&param));

        let param: FrameParam<String> = FrameParam {
            action_duration: Some(1.2),
            want_move: 0.0,
            ..Default::default()
        };
        assert!(!exit_logic.should_exit(&param));

        let param: FrameParam<String> = FrameParam {
            action_duration: Some(1.2),
            want_move: 1.0,
            ..Default::default()
        };
        assert!(!exit_logic.should_exit(&param));

        let param: FrameParam<String> = FrameParam {
            action_duration: Some(1.3),
            want_move: 0.0,
            ..Default::default()
        };
        assert!(!exit_logic.should_exit(&param));

        let param: FrameParam<String> = FrameParam {
            action_duration: Some(1.3),
            want_move: 1.0,
            ..Default::default()
        };
        assert!(exit_logic.should_exit(&param));
    }

    #[test]
    fn exit_logic_jump_after() {
        let exit_logic = MovementActionExitLogic::ExitLogic(ActionBaseExitLogic::JumpAfter(1.2));

        let param: FrameParam<String> = FrameParam {
            action_duration: Some(1.3),
            want_jump: false,
            ..Default::default()
        };
        assert!(!exit_logic.should_exit(&param));

        let param: FrameParam<String> = FrameParam {
            action_duration: Some(1.3),
            want_jump: true,
            ..Default::default()
        };
        assert!(exit_logic.should_exit(&param));
    }
}

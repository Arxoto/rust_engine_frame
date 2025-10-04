//! 运动模式对【事件】和【退出逻辑】的功能增强

use crate::{
    cores::unify_type::FixedString,
    motion::{
        action_impl::{ActionBaseEvent, ActionBaseExitLogic},
        action_types::{ActionEvent, ActionExitLogic},
        movement_impl::MovementMode,
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
pub enum MovementActionExitLogic {
    ExitLogic(ActionBaseExitLogic),
    MovementChange(MovementMode, MovementMode),
}

impl MovementActionExitLogic {
    fn should_exit_by_logic<S: FixedString>(
        exit_logic: &ActionBaseExitLogic,
        param: &FrameParam<S>,
    ) -> bool {
        match exit_logic {
            ActionBaseExitLogic::AnimFinished => param.anim_finished,
            ActionBaseExitLogic::MoveAfter(the_time) => {
                param.action_duration > *the_time && param.want_move
            }
            ActionBaseExitLogic::JumpAfter(the_time) => {
                param.action_duration > *the_time && param.want_jump
            }
        }
    }

    fn should_exit_by_movement_change<S: FixedString>(
        old_movement: &MovementMode,
        new_movement: &MovementMode,
        param: &FrameParam<S>,
    ) -> bool {
        match param.movement_changed {
            (Some(the_old), Some(the_new)) => the_old == *old_movement && the_new == *new_movement,
            _ => false,
        }
    }
}

impl<S: FixedString> ActionExitLogic<FrameParam<S>> for MovementActionExitLogic {
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

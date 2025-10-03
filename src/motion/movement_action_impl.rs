//! 运动模式对【事件】和【退出逻辑】的功能增强

use crate::motion::{
    action_impl::{ActionBaseEvent, ActionBaseExitLogic},
    action_types::{ActionEvent, ActionExitLogic},
    movement_impl::MovementMode,
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

pub struct ActionMovementExitParam {
    pub(crate) movement_changed: (Option<MovementMode>, Option<MovementMode>),
}

impl ActionExitLogic for MovementActionExitLogic {
    type ExitParam = ActionMovementExitParam;

    fn should_exit(&self, _p: &Self::ExitParam) -> bool {
        todo!()
    }
}

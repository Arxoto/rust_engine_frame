//! 运动模式对【事件】和【退出逻辑】的实现和增强

use crate::{
    cores::unify_type::FixedString,
    motions::{
        abstracts::{
            action_types::{ActionEvent, ActionExitLogic},
            player_input::PlayerOperation,
        },
        motion_mode::MotionMode,
        state_machine_phy_param::PhyParam,
    },
};

/// 动作的触发
/// - 指令 Instruction （指令用作 combo 时建议主要在 [`ActionBaseExitLogic`] 中实现）
/// - 信号 Signal
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ActionBaseEvent {
    /// 跳跃指令
    JumpInstruction,
    /// 跳跃指令（长摁跳更高）
    JumpHigherInstruction,
    /// 闪避指令
    DodgeInstruction,
    /// 防御指令
    BlockInstruction,
    /// 攻击指令
    AttackInstruction,
    /// 攻击指令（长摁重击）
    AttackHeavierInstruction,

    /// 命中对方
    ///
    /// - 若想实现【命中后自动连击】或者【命中后才能衔接】，则在接收该信号后转换状态，状态的第一个动画为同名动画，因此不会导致动画提前取消
    HitSignal,
    /// 被命中
    ///
    /// - 若想实现【受击后自动格挡】，则立刻切换状态
    BeHitSignal,
}

impl ActionEvent for ActionBaseEvent {}

/// 和 [`MotionMode`] 组合来实现 [`ActionEvent`]
///
/// 支持仅某种 运动状态 下的事件触发
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
            .iter()
            .map(|motion| Self::new(event, *motion))
            .collect()
    }
}

/// 动作的退出逻辑
///
/// impl in [`MotionActionExitLogic`]
#[derive(Clone, Copy, Debug)]
pub enum ActionBaseExitLogic<S: FixedString> {
    /// 动画结束播放
    AnimFinished(S),
    /// 多长时间后，移动可取消后摇
    MoveAfter(f64),
    /// 多长时间后，跳跃可打断
    JumpAfter(f64),
    /// 当播放动画为S时，攻击可打断
    ///
    /// 同时可实现攻击指令的预输入：切换到首动画为同名动画的动作
    AttackWhen(S),
}

impl<S: FixedString> ActionBaseExitLogic<S> {
    pub(crate) fn should_exit_by_logic(&self, param: &PhyParam<S>) -> bool {
        match self {
            ActionBaseExitLogic::AnimFinished(anim_name) => {
                param.anim_finished && param.anim_name == *anim_name
            }
            ActionBaseExitLogic::MoveAfter(the_time) => {
                param.instructions.move_direction.op_active()
                    && param
                        .inner_param
                        .action_duration
                        .map(|duration_time| duration_time > *the_time)
                        .unwrap_or(false)
            }
            ActionBaseExitLogic::JumpAfter(the_time) => {
                param.instructions.jump_once.op_active()
                    && param
                        .inner_param
                        .action_duration
                        .map(|duration_time| duration_time > *the_time)
                        .unwrap_or(false)
            }
            ActionBaseExitLogic::AttackWhen(anim_name) => {
                param.instructions.attack_once.op_active() && param.anim_name == *anim_name
            }
        }
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
    MotionOnlyAllowed(MotionMode),
}

impl<S: FixedString> ActionExitLogic<PhyParam<S>> for MotionActionExitLogic<S> {
    fn should_exit(&self, exit_param: &PhyParam<S>) -> bool {
        match self {
            MotionActionExitLogic::ExitLogic(exit_logic) => {
                exit_logic.should_exit_by_logic(exit_param)
            }
            MotionActionExitLogic::MotionOnlyAllowed(allowed_motion) => {
                match exit_param.inner_param.motion_changed {
                    Some((_, current_motion)) => current_motion != *allowed_motion,
                    None => false,
                }
            }
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::motions::{
        abstracts::player_pre_input::PreInputInstruction,
        player_controller::PlayerInstructionCollection, state_machine_phy_param::PhyInnerParam,
    };

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
            inner_param: PhyInnerParam {
                action_duration: Some(1.0),
                ..Default::default()
            },
            instructions: PlayerInstructionCollection {
                move_direction: 0.0.into(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(!exit_logic.should_exit(&param));

        let param: PhyParam<String> = PhyParam {
            inner_param: PhyInnerParam {
                action_duration: Some(1.0),
                ..Default::default()
            },
            instructions: PlayerInstructionCollection {
                move_direction: 1.0.into(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(!exit_logic.should_exit(&param));

        let param: PhyParam<String> = PhyParam {
            inner_param: PhyInnerParam {
                action_duration: Some(1.2),
                ..Default::default()
            },
            instructions: PlayerInstructionCollection {
                move_direction: 0.0.into(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(!exit_logic.should_exit(&param));

        let param: PhyParam<String> = PhyParam {
            inner_param: PhyInnerParam {
                action_duration: Some(1.2),
                ..Default::default()
            },
            instructions: PlayerInstructionCollection {
                move_direction: 1.0.into(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(!exit_logic.should_exit(&param));

        let param: PhyParam<String> = PhyParam {
            inner_param: PhyInnerParam {
                action_duration: Some(1.3),
                ..Default::default()
            },
            instructions: PlayerInstructionCollection {
                move_direction: 0.0.into(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(!exit_logic.should_exit(&param));

        let param: PhyParam<String> = PhyParam {
            inner_param: PhyInnerParam {
                action_duration: Some(1.3),
                ..Default::default()
            },
            instructions: PlayerInstructionCollection {
                move_direction: 1.0.into(),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(exit_logic.should_exit(&param));
    }

    #[test]
    fn exit_logic_jump_after() {
        let exit_logic = MotionActionExitLogic::ExitLogic(ActionBaseExitLogic::JumpAfter(1.2));

        let param: PhyParam<String> = PhyParam {
            inner_param: PhyInnerParam {
                action_duration: Some(1.3),
                ..Default::default()
            },
            instructions: PlayerInstructionCollection {
                jump_once: PreInputInstruction(false, Default::default()),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(!exit_logic.should_exit(&param));

        let param: PhyParam<String> = PhyParam {
            inner_param: PhyInnerParam {
                action_duration: Some(1.3),
                ..Default::default()
            },
            instructions: PlayerInstructionCollection {
                jump_once: PreInputInstruction(true, Default::default()),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(exit_logic.should_exit(&param));
    }
}

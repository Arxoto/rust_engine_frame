//! 【动作】和【行为】和【玩家角色】的状态机的依赖类定义

use crate::{
    cores::unify_type::FixedString,
    motion::{
        action::Action,
        behaviour::Behaviour,
        movement_action_impl::{MovementActionEvent, MovementActionExitLogic},
        movement_impl::MovementMode,
    },
};

#[derive(Default)]
pub struct FrameParam<S: FixedString> {
    // 客观
    pub delta: f64,
    pub anim_finished: bool,
    pub anim_name: S, // 外部传入 因为考虑到动画不一定完全由动作系统控制
    // 意图
    pub want_move: bool,
    pub want_jump: bool,
    // 框架内部维护
    pub(crate) movement_changed: (Option<MovementMode>, Option<MovementMode>),
    pub(crate) action_duration: f64, // todo 这里需要思考这里是否修改为 Option 类型，考虑到：1、内部维护，不从外界传入，明确状态；2、单次消费掉后就不可用（涉及一帧内的状态修改）
}

pub struct PhyParam<S: FixedString> {
    pub delta: f64,
    pub anim_name: S,
}

/// ExitParam 为 FrameParam ，角色状态机将输入参数聚合成一个
pub type MovementAction<S, PhyEff> =
    Action<S, MovementActionEvent, FrameParam<S>, MovementActionExitLogic<S>, PhyEff>;

/// EnterParam 为 FrameParam ，角色状态机将输入参数聚合成一个
pub trait MovementBehaviour<S: FixedString, FrameEff, PhyEff>:
    Behaviour<FrameParam<S>, FrameParam<S>, FrameEff, PhyParam<S>, PhyEff>
{
    fn get_movement_mode(&self) -> MovementMode;
}

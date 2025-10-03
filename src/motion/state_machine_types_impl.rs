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

pub struct EnterParam;

pub struct FrameParam<S: FixedString> {
    pub delta: f64,
    pub anim_name: S,
    pub anim_finished: bool,
}

pub struct PhyParam<S: FixedString> {
    pub delta: f64,
    pub anim_name: S,
}

pub type MovementAction<S, PhyEff> =
    Action<S, MovementActionEvent, MovementActionExitLogic, PhyEff>;

pub trait MovementBehaviour<S: FixedString, FrameEff, PhyEff>:
    Behaviour<EnterParam, FrameParam<S>, FrameEff, PhyParam<S>, PhyEff>
{
    fn get_movement_mode(&self) -> MovementMode;
}

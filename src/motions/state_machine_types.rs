//! 【动作】和【行为】和【玩家角色】的状态机的依赖类定义
//!
//! 实现为2D游戏，因为输入参数和输出效果根据2D3D均有差别，索性一起抽象
//!
//! （虽然仅行为系统的实现与2D3D有关，但是仍然不独立抽象一层，因为输入参数连带着也要抽象一层特征，暂时没必要）

use crate::{
    cores::unify_type::FixedString,
    motions::{
        abstracts::{action::Action, behaviour::Behaviour},
        motion_action::{MotionActionEvent, MotionActionExitLogic},
        state_machine_frame_eff::FrameEff,
        state_machine_frame_param::FrameParam,
        state_machine_phy_eff::{MotionData, PhyEff},
        state_machine_phy_param::PhyParam,
    },
};

/// ExitParam 为 FrameParam ，角色状态机将输入参数聚合成一个
pub type MotionAction<S, PhyEff> =
    Action<S, MotionActionEvent, PhyParam<S>, MotionActionExitLogic<S>, PhyEff>;

/// EnterParam 为 [`FrameParam`] ，角色状态机将输入参数聚合成一个
pub trait MotionBehaviour<S: FixedString, FrameEff, PhyEff>:
    for<'a> Behaviour<
        PhyParam<S>,
        FrameParam<S>,
        FrameEff,
        (&'a mut PhyParam<S>, &'a MotionData),
        PhyEff,
    >
{
}

/// 最终效果聚合器 将两个状态机的结果聚合
pub trait EffGenerator<S: FixedString, FE, PE> {
    fn gen_frame_eff(by_action: &S, by_behaviour: Option<FrameEff<S>>) -> FE;
    fn gen_phy_eff(by_action: Option<PhyEff>, by_behaviour: Option<PhyEff>) -> PE;
}

pub struct CommonEffGenerator;
impl<S: FixedString>
    EffGenerator<S, (Option<FrameEff<S>>, Option<FrameEff<S>>), (Option<PhyEff>, Option<PhyEff>)>
    for CommonEffGenerator
{
    fn gen_frame_eff(
        by_action: &S,
        by_behaviour: Option<FrameEff<S>>,
    ) -> (Option<FrameEff<S>>, Option<FrameEff<S>>) {
        (FrameEff::try_new(by_action.clone()), by_behaviour)
    }

    fn gen_phy_eff(
        by_action: Option<PhyEff>,
        by_behaviour: Option<PhyEff>,
    ) -> (Option<PhyEff>, Option<PhyEff>) {
        (by_action, by_behaviour)
    }
}

/// by_action first, and then by_behaviour
pub struct ActionBehaviourGenerator;
impl<S: FixedString> EffGenerator<S, Option<FrameEff<S>>, Option<PhyEff>>
    for ActionBehaviourGenerator
{
    fn gen_frame_eff(by_action: &S, by_behaviour: Option<FrameEff<S>>) -> Option<FrameEff<S>> {
        FrameEff::try_new(by_action.clone()).or(by_behaviour)
    }

    fn gen_phy_eff(by_action: Option<PhyEff>, by_behaviour: Option<PhyEff>) -> Option<PhyEff> {
        by_action.or(by_behaviour)
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn gen_frame_eff() {
        let eff = ActionBehaviourGenerator::gen_frame_eff(&"111", Some(FrameEff::from("222")));
        assert_eq!(eff.unwrap().anim_name, "111");

        let eff = ActionBehaviourGenerator::gen_frame_eff(&"", Some(FrameEff::from("anim")));
        assert_eq!(eff.unwrap().anim_name, "anim");
    }
}

use crate::{
    cores::unify_type::FixedString,
    motions::{
        abstracts::behaviour::Behaviour,
        motion_mode::MotionMode,
        state_machine_frame_eff::FrameEff,
        state_machine_frame_param::FrameParam,
        state_machine_phy_eff::{MotionData, PhyEff},
        state_machine_phy_param::PhyParam,
        state_machine_types::MotionBehaviour,
    },
};

const SELF_MOTION_MODE: MotionMode = MotionMode::Motionless;

/// 行为系统的一般实现 无法移动 一般用作强制状态切换
///
/// 也可作为最小实现 用作模板创建新行为
#[derive(Debug, Default)]
pub struct CommonBehaviour<S: FixedString> {
    the_anim: S,
}

impl<S: FixedString> CommonBehaviour<S> {
    pub fn new() -> Self {
        Self {
            the_anim: S::default(),
        }
    }
}

impl<S: FixedString>
    Behaviour<PhyParam<S>, FrameParam<S>, FrameEff<S>, (&mut PhyParam<S>, &MotionData), PhyEff>
    for CommonBehaviour<S>
{
    fn will_enter(&self, p: &PhyParam<S>) -> bool {
        match p.inner_param.motion_state {
            Some((_, mode)) => mode == SELF_MOTION_MODE,
            None => false,
        }
    }

    fn on_enter(&mut self, _p: &PhyParam<S>) {
        // do something
    }

    fn tick_frame(&mut self, _p: &FrameParam<S>) -> FrameEff<S> {
        FrameEff::from(self.the_anim.clone())
    }

    fn process_physics(&mut self, (p, data): &mut (&mut PhyParam<S>, &MotionData)) -> PhyEff {
        PhyEff::create_force_stop(data, p.instructions.move_direction.0)
    }
}

impl<S: FixedString> MotionBehaviour<S, FrameEff<S>, PhyEff> for CommonBehaviour<S> {}

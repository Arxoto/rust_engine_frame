use crate::{
    cores::{tiny_timer::TinyTimer, unify_type::FixedString},
    motions::{
        abstracts::{behaviour::Behaviour, player_pre_input::PreInputOperation},
        motion_mode::MotionMode,
        state_machine_frame_eff::FrameEff,
        state_machine_frame_param::FrameParam,
        state_machine_phy_eff::{MotionData, PhyEff},
        state_machine_phy_param::PhyParam,
        state_machine_types::MotionBehaviour,
    },
};

const SELF_MOTION_MODE: MotionMode = MotionMode::ClimbWall;
const CLIMB_BEGIN_TIME: f64 = 0.2;

/// 攀爬（贴墙下滑）
#[derive(Debug, Default)]
pub struct ClimbWallBehaviour<S: FixedString> {
    climb_begin_anim: S,
    climbing_anim: S,
    beginning: TinyTimer,
}

impl<S: FixedString> ClimbWallBehaviour<S> {
    pub fn new(climb_begin_anim: S, climbing_anim: S) -> Self {
        Self {
            climb_begin_anim,
            climbing_anim,
            beginning: TinyTimer::new(CLIMB_BEGIN_TIME),
        }
    }
}

impl<S: FixedString>
    Behaviour<PhyParam<S>, FrameParam<S>, FrameEff<S>, (&mut PhyParam<S>, &MotionData), PhyEff>
    for ClimbWallBehaviour<S>
{
    fn will_enter(&self, p: &PhyParam<S>) -> bool {
        match p.inner_param.motion_state {
            Some((_, mode)) => mode == SELF_MOTION_MODE,
            None => false,
        }
    }

    fn on_enter(&mut self, _p: &PhyParam<S>) {
        self.beginning.start_time();
    }

    fn tick_frame(&mut self, p: &FrameParam<S>) -> FrameEff<S> {
        self.beginning.add_time(p.delta);
        // 攀爬没有跳跃动画 因为预期内跳跃会切换至另一个行为
        if self.beginning.in_time() {
            FrameEff::from(self.climb_begin_anim.clone())
        } else {
            FrameEff::from(self.climbing_anim.clone())
        }
    }

    fn process_physics(&mut self, (p, data): &mut (&mut PhyParam<S>, &MotionData)) -> PhyEff {
        if p.instructions.jump_once.op_consume_active() {
            PhyEff::create_jump(data, p.instructions.move_direction.0)
        } else {
            PhyEff::create_climb(data, p.instructions.move_direction.0)
        }
    }
}

impl<S: FixedString> MotionBehaviour<S, FrameEff<S>, PhyEff> for ClimbWallBehaviour<S> {}

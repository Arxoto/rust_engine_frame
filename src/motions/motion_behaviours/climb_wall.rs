use crate::{
    cores::{tiny_timer::TinyTimer, unify_type::FixedString},
    motions::{
        abstracts::{behaviour::Behaviour, player_pre_input::PreInputOperation},
        motion_mode::MotionMode,
        state_machine_frame_eff::FrameEff,
        state_machine_param::{FrameParam, PhyParam},
        state_machine_phy_eff::{MotionData, PhyEff},
        state_machine_types::MotionBehaviour,
    },
};

const CLIMB_BEGIN_TIME: f64 = 0.2;

/// 攀爬（贴墙下滑）
#[derive(Debug, Default)]
pub struct ClimbWallBehaviour<S: FixedString> {
    pub(crate) climb_begin_anim: S,
    pub(crate) climbing_anim: S,
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
        p.character_can_climb // 判断条件需要用到向量运算 为保证项目纯净 交由外部判断输入
    }

    fn on_enter(&mut self) {
        self.beginning.start_time();
    }

    fn on_exit(&mut self) {}

    fn tick_frame(&mut self, p: &FrameParam<S>) -> FrameEff<S> {
        self.beginning.add_time(p.delta);
        if self.beginning.in_time() {
            FrameEff::from(self.climb_begin_anim.clone())
        } else {
            FrameEff::from(self.climbing_anim.clone())
        }
    }

    fn process_physics(&mut self, (p, data): &mut (&mut PhyParam<S>, &MotionData)) -> PhyEff {
        if p.jump_once.op_consume_active() {
            PhyEff::create_jump(data, p.move_direction.0)
        } else {
            PhyEff::create_climb(data, p.move_direction.0)
        }
    }
}

impl<S: FixedString> MotionBehaviour<S, FrameEff<S>, PhyEff> for ClimbWallBehaviour<S> {
    fn get_motion_mode(&self) -> MotionMode {
        MotionMode::ClimbWall
    }
}

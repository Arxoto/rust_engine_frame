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

const RUN_OR_IDLE_THRESHOLD: f64 = 0.1;
const LANDING_DELAY: f64 = 0.1;

/// 地面
#[derive(Debug, Default)]
pub struct OnFloorBehaviour<S: FixedString> {
    run_anim: S,
    idle_anim: S,
    landing_anim: S,
    landing_timer: TinyTimer,
}

impl<S: FixedString> OnFloorBehaviour<S> {
    pub fn new(run_anim: S, idle_anim: S, landing_anim: S) -> Self {
        Self {
            run_anim,
            idle_anim,
            landing_anim,
            landing_timer: TinyTimer::new(LANDING_DELAY),
        }
    }
}

impl<S: FixedString>
    Behaviour<PhyParam<S>, FrameParam<S>, FrameEff<S>, (&mut PhyParam<S>, &MotionData), PhyEff>
    for OnFloorBehaviour<S>
{
    fn will_enter(&self, p: &PhyParam<S>) -> bool {
        p.character_is_on_floor
    }

    fn on_enter(&mut self) {}

    fn on_exit(&mut self) {}

    fn tick_frame(&mut self, p: &FrameParam<S>) -> FrameEff<S> {
        // 着陆缓冲动画（仅视觉效果）
        self.landing_timer.add_time(p.delta);
        if p.character_landing {
            self.landing_timer.start_time();
        }
        if self.landing_timer.in_time() {
            return FrameEff::from(self.landing_anim.clone());
        }

        if p.character_x_velocity.abs() < RUN_OR_IDLE_THRESHOLD {
            FrameEff::from(self.idle_anim.clone())
        } else {
            FrameEff::from(self.run_anim.clone())
        }
    }

    fn process_physics(&mut self, (p, data): &mut (&mut PhyParam<S>, &MotionData)) -> PhyEff {
        // hard-landing 硬着陆眩晕效果通过动作系统实现（或者说一切非自由移动的状态都能通过动作系统实现）
        if p.jump_once.op_consume_active() {
            // jump immediately 操作连贯
            return PhyEff::create_jump(data, p.move_direction.0);
        }
        PhyEff::create_run(data, p.move_direction.0)
    }
}

impl<S: FixedString> MotionBehaviour<S, FrameEff<S>, PhyEff> for OnFloorBehaviour<S> {
    fn get_motion_mode(&self) -> MotionMode {
        MotionMode::FreeStat
    }
}

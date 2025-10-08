use crate::{
    cores::{tiny_timer::TinyTimer, unify_type::FixedString},
    motions::{
        abstracts::{
            behaviour::Behaviour, player_input::PlayerOperation,
            player_pre_input::PreInputOperation,
        },
        motion_mode::MotionMode,
        state_machine_frame_eff::FrameEff,
        state_machine_param::{FrameParam, PhyParam},
        state_machine_phy_eff::{MotionData, PhyEff},
        state_machine_types::MotionBehaviour,
    },
};

const COYOTE_TIME_DELAY: f64 = 0.1;

/// 跳跃的特效 在视觉上区分特殊的跳跃
#[derive(Debug)]
enum JumpSpecialEffect {
    None,
    DoubleJump, // 可以支持次数以区分多段跳跃
    JumpOnWall,
}

impl Default for JumpSpecialEffect {
    fn default() -> Self {
        Self::None
    }
}

/// 空中行为
#[derive(Debug, Default)]
pub struct InAirBehaviour<S: FixedString> {
    jumping_anim: S,
    falling_anim: S,
    jump_on_wall_eff: S,
    double_jump_eff: S,

    coyote_timer: TinyTimer,
    jump_higher_timer: TinyTimer,
    double_jump: i64,
    double_jump_value: i64,

    //tmp
    jump_special_effect: JumpSpecialEffect,
}

impl<S: FixedString> InAirBehaviour<S> {
    pub fn new(
        jumping_anim: S,
        falling_anim: S,
        jump_on_wall_eff: S,
        double_jump_eff: S,
        jump_higher_delay: f64,
        double_jump_value: i64,
    ) -> Self {
        Self {
            jumping_anim,
            falling_anim,
            jump_on_wall_eff,
            double_jump_eff,
            coyote_timer: TinyTimer::new(COYOTE_TIME_DELAY),
            jump_higher_timer: TinyTimer::new(jump_higher_delay),
            double_jump: 0,
            double_jump_value,
            jump_special_effect: Default::default(),
        }
    }

    fn start_double_jump_time(&mut self) {
        self.double_jump = 0
    }

    fn can_double_jump(&self) -> bool {
        self.double_jump < self.double_jump_value
    }

    fn add_double_jump_time(&mut self) {
        self.double_jump = self.double_jump_value.min(self.double_jump + 1);
    }
}

impl<S: FixedString>
    Behaviour<PhyParam<S>, FrameParam<S>, FrameEff<S>, (&mut PhyParam<S>, &MotionData), PhyEff>
    for InAirBehaviour<S>
{
    fn will_enter(&self, p: &PhyParam<S>) -> bool {
        !p.character_is_on_floor
    }

    fn on_enter(&mut self, p: &PhyParam<S>) {
        self.start_double_jump_time();

        if p.character_y_fly_up {
            self.coyote_timer.final_time();
            self.jump_higher_timer.start_time();
        } else {
            self.coyote_timer.start_time();
            self.jump_higher_timer.final_time();
        }
    }

    fn tick_frame(&mut self, p: &FrameParam<S>) -> FrameEff<S> {
        let special_eff = match self.jump_special_effect {
            JumpSpecialEffect::None => S::default(),
            JumpSpecialEffect::DoubleJump => self.double_jump_eff.clone(),
            JumpSpecialEffect::JumpOnWall => self.jump_on_wall_eff.clone(),
        };
        self.jump_special_effect = JumpSpecialEffect::None;

        if p.character_y_fly_up {
            FrameEff {
                anim_name: self.jumping_anim.clone(),
                special_eff,
            }
        } else {
            FrameEff {
                anim_name: self.falling_anim.clone(),
                special_eff,
            }
        }
    }

    fn process_physics(&mut self, (p, data): &mut (&mut PhyParam<S>, &MotionData)) -> PhyEff {
        self.coyote_timer.add_time(p.delta);
        if p.character_can_jump_on_wall {
            self.coyote_timer.start_time();
        }
        self.jump_higher_timer.add_time(p.delta);

        // 处理跳跃下落逻辑
        if p.jump_once.op_active() {
            // 尝试跳跃 注意尝试失败也不会消耗该指令
            let should_jump_once = if p.character_can_jump_on_wall {
                self.jump_special_effect = JumpSpecialEffect::JumpOnWall;
                true
            } else if self.coyote_timer.in_time() {
                self.coyote_timer.final_time();
                true
            } else if self.can_double_jump() {
                self.jump_special_effect = JumpSpecialEffect::DoubleJump;
                self.add_double_jump_time();
                true
            } else {
                false
            };

            if should_jump_once {
                p.jump_once.op_echo();
                self.jump_higher_timer.start_time();
                return PhyEff::create_jump(data, p.move_direction.0);
            }
        } else if p.jump_keep.op_active() {
            // 尝试跳得更高
            if self.jump_higher_timer.in_time() {
                return PhyEff::create_jumping(data, p.move_direction.0);
            }
        } else {
            // 中断任何跳跃意图都会导致无法继续跳得更高
            self.jump_higher_timer.final_time();
        }

        PhyEff::create_falling(data, p.move_direction.0)
    }
}

impl<S: FixedString> MotionBehaviour<S, FrameEff<S>, PhyEff> for InAirBehaviour<S> {
    fn get_motion_mode(&self) -> MotionMode {
        MotionMode::InAir
    }
}

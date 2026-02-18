use crate::{
    cores::{tiny_timer::TinyTimer, unify_type::FixedString},
    motions::{
        abstracts::{
            behaviour::Behaviour, player_input::PlayerOperation,
            player_pre_input::PreInputOperation,
        },
        motion_mode::MotionMode,
        state_machine_frame_eff::FrameEff,
        state_machine_frame_param::FrameParam,
        state_machine_phy_eff::{MotionData, PhyEff},
        state_machine_phy_param::PhyParam,
        state_machine_types::MotionBehaviour,
    },
};

const SELF_MOTION_MODE: MotionMode = MotionMode::OnFloor;
const LANDING_DELAY: f64 = 0.1;
const MOVEING_THRESHOLD: f64 = 0.1; // 视觉相关 没必要很小

/// 地面
#[derive(Debug, Default)]
pub struct OnFloorBehaviour<S: FixedString> {
    run_anim: S,
    idle_anim: S,
    landing_anim: S,
    landing_timer: TinyTimer,
    ready_jump_anim: S,
    ready_jump_timer: TinyTimer,
    turn_back_anim: S,
    turn_back_flag: bool,
    turn_back_velocity: f64,
}

impl<S: FixedString> OnFloorBehaviour<S> {
    pub fn new(
        run_anim: S,
        idle_anim: S,
        landing_anim: S,
        ready_jump_anim: S,
        jump_delay: f64,
        turn_back_anim: S,
        turn_back_velocity: f64,
    ) -> Self {
        Self {
            run_anim,
            idle_anim,
            landing_anim,
            landing_timer: TinyTimer::new(LANDING_DELAY),
            ready_jump_anim,
            ready_jump_timer: TinyTimer::new(jump_delay),
            turn_back_anim,
            turn_back_flag: false,
            turn_back_velocity,
        }
    }
}

impl<S: FixedString>
    Behaviour<PhyParam<S>, FrameParam<S>, FrameEff<S>, (&mut PhyParam<S>, &MotionData), PhyEff>
    for OnFloorBehaviour<S>
{
    fn will_enter(&self, p: &PhyParam<S>) -> bool {
        match p.inner_param.motion_state {
            Some((_, mode)) => mode == SELF_MOTION_MODE,
            None => false,
        }
    }

    fn on_enter(&mut self, p: &PhyParam<S>) {
        if p.character_landing {
            self.landing_timer.start_time();
        } else {
            self.landing_timer.final_time();
        }
        self.ready_jump_timer.final_time();
    }

    fn tick_frame(&mut self, p: &FrameParam<S>) -> FrameEff<S> {
        // 着陆缓冲动画（仅视觉效果）
        self.landing_timer.add_time(p.delta);
        if self.landing_timer.in_time() {
            return FrameEff::from(self.landing_anim.clone());
        }

        if self.ready_jump_timer.in_time() {
            // 若资源充足可制作跑步起跳动画
            return FrameEff::from(self.ready_jump_anim.clone());
        }

        if self.turn_back_flag || p.anim_playing(&self.turn_back_anim) {
            // 外界控制动画优先级 内部不好判断
            return FrameEff {
                anim_name: self.turn_back_anim.clone(),
                special_eff: Default::default(),
                not_turn_back: true,
            };
        }

        if p.character_x_velocity.abs() > MOVEING_THRESHOLD {
            FrameEff::from(self.run_anim.clone())
        } else {
            FrameEff::from(self.idle_anim.clone())
        }
    }

    fn process_physics(&mut self, (p, data): &mut (&mut PhyParam<S>, &MotionData)) -> PhyEff {
        // 每帧判断 是否转身
        self.turn_back_flag = p.want_turn_back(self.turn_back_velocity);

        // hard-landing 硬着陆眩晕效果通过动作系统实现（或者说一切非自由移动的状态都能通过动作系统实现）
        self.ready_jump_timer.add_time(p.delta);
        if p.instructions.jump_once.op_active() {
            // 起跳动画 不要立即跳跃（缺乏重量感和冲击力） 不要过长（不跟手）
            // 快节奏/硬核 0.05 - 0.15 秒
            // 中节奏/流畅 0.15 - 0.2  秒
            // 慢节奏/蓄力 0.2  - 0.5  秒 （超过 0.25 秒会有严重的输入延迟）
            // 注意期间若离开平台 通过郊狼时间实现跳跃 （真正起跳时才进行回响）
            if !self.ready_jump_timer.in_time() {
                self.ready_jump_timer.start_time();
            }
        }
        if self.ready_jump_timer.is_end() {
            p.instructions.jump_once.op_do_deactivate();
            return PhyEff::create_jump(data, p.instructions.move_direction.0);
        }
        PhyEff::create_run(data, p.instructions.move_direction.0)
    }
}

impl<S: FixedString> MotionBehaviour<S, FrameEff<S>, PhyEff> for OnFloorBehaviour<S> {}

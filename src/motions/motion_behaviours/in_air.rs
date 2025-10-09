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

    /// 问题：边缘无法跳跃
    /// 分析：边缘之外才尝试跳跃，此时因为在空中无法进行跳跃
    /// 预期：刚走出边缘的一段时间内允许跳跃
    /// 分析：定义走出边缘：不是通过跳跃进入空中的
    /// 实现：检测上一帧是否尝试跳跃，基本等价于检测这一帧有无向上速度（因此在 on_enter 中使用y轴是否向上做判断）
    /// 例外：跳跃了但无速度：上一帧跳跃但是碰撞，本帧无碰撞仍然可跳（无所谓）
    /// 例外：没跳跃但有速度：上一帧非主观原因导致升空，此时仍然可跳，存在逻辑错误（非主观升空即【不可控状态】，通过动作系统覆盖实现，而郊狼时间一般较短，因此无影响）
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
        // 提示，从攀爬状态转换到空中状态时触发的是普通的郊狼时间
        if p.character_can_jump_on_wall {
            self.coyote_timer.start_time();
        }
        self.jump_higher_timer.add_time(p.delta);

        // 处理跳跃下落逻辑
        if p.instructions.jump_once.op_active() {
            // 尝试跳跃 注意尝试失败也不会消耗该指令
            let should_jump_once = if p.character_can_jump_on_wall {
                // jump_on_wall 优化攀爬时体验（未进入攀爬状态，脚部碰撞墙体但手部没有碰撞，此时可以直接跳跃）
                // 扩展能力：伸出式平台（壁架）边缘呈倒阶梯状，操作得当时可以逆攀而上
                // P.S. 每个台阶二高度（允许脚部碰撞墙体）不依赖二段跳，一高度依赖二段跳（若蹬墙跳要求仅为身体与墙碰撞，则极限操作下也可以完成）
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
                p.instructions.jump_once.op_echo();
                self.jump_higher_timer.start_time();
                return PhyEff::create_jump(data, p.instructions.move_direction.0);
            }
        } else if p.instructions.jump_keep.op_active() {
            // 尝试跳得更高
            if self.jump_higher_timer.in_time() {
                return PhyEff::create_jumping(data, p.instructions.move_direction.0);
            }
        } else {
            // 中断任何跳跃意图都会导致无法继续跳得更高
            self.jump_higher_timer.final_time();
        }

        PhyEff::create_falling(data, p.instructions.move_direction.0)
    }
}

impl<S: FixedString> MotionBehaviour<S, FrameEff<S>, PhyEff> for InAirBehaviour<S> {
    fn get_motion_mode(&self) -> MotionMode {
        MotionMode::InAir
    }
}

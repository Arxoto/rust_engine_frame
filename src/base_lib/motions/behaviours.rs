//! 控制一个动作内部具体的行为
//!
//! 这里仅作样例，具体应该在业务侧根据策划需求去实现

/// 抽象行为，定义特征/接口/虚类
///
/// 部分复杂的动作需要行为进行拓展，一个动作可对应零或一个行为，复杂行为可在内部细分
///
/// - on_enter
/// - on_exit
/// - tick 帧逻辑，一般在 _physics_process / FixedUpdate 中调用
mod abstract_behaviour {}

/// 基础行为，存放通用逻辑，无论什么行为都会去调用
///
/// 识别【客观条件】，自动更新【运动模式】 tag
///
/// - climb_wall
/// - in_air
/// - on_land
/// - under_water
mod base_behaviour {}

/// 依赖特定 tag 才能进入的高优先级动作行为，不会切换至其他状态，打开上帝模式在测试时用
mod god_behaviour {}

/// 跳跃一般行为
///
/// 业务逻辑参考跳跃优先级： 踩踏跳 > 蹬墙跳 > 郊狼时间跳跃 > 二段跳
///
/// - stamp_jump 踩踏跳，跳跃能力增强
///   - 脚触碰到敌人时重置为【未跳跃状态】
///   - 脚触碰到敌人时跳跃触发，附带冲击力
/// - wall_jump 蹬墙跳，跳跃能力增强
///   - 脚触碰到墙面时重置为【未跳跃状态】
///   - 脚触碰到墙面时跳跃触发，与普通跳跃的灰尘特效方向相反
///   - 应只允许部分墙面才可借力跳跃
///   - 手脚都碰墙则进入爬墙状态，只有脚碰到视为蹬墙跳
///   - 扩展能力：伸出的平台（壁架）边缘呈倒阶梯状，操作得当时可以逆攀而上
///     - 每个台阶二高度（允许脚部碰撞墙体）不依赖二段跳
///     - 每个台阶一高度依赖二段跳（若允许引体向上模拟跳跃，则极限操作下也可以完成）
/// - coyote_time 郊狼时间，跳跃操作体验优化
///   - 郊狼时间内可触发跳跃
/// - double_jump 二段跳，跳跃能力增强
///   - 业务自己实现，辅助工具只能在【未跳跃状态】时自动重置二段跳次数，不能在“跳跃至空中”的场景下实现自动重置
/// - higher_jump
///   - 进入【大跳状态】后持续判断是否【不间断摁住】跳跃键以执行大跳行为
///   - 期间上升速度恒定或者重力影响小，结束后正常受重力影响
///
/// 每次进入空中状态时，检测上一帧是否尝试跳跃，基本等价于检测这一帧有无向上速度
/// - 例外，跳跃了但无速度：上一帧跳跃但是碰撞导致失败了，本帧可以通过郊狼时间和预输入缓冲成功触发跳跃，因此反而符合预期
/// - 例外，没跳跃但有速度：上一帧非主观原因导致升空，此时仍然可跳，存在逻辑错误
///   - 非主观升空即【不可控状态】，通过动作系统覆盖，而郊狼时间一般较短，因此判断无影响
pub mod in_air_behaviour {
    use crate::base_lib::cores::{
        tick_timer::TickTimerFinite,
        tiny_timer::{FlowingTimer, FlowingTimerReadonly, TickTimer},
    };

    // 不适合使用 类型状态模式 (Type_State_Pattern) ，因为需要被传递，类型都是编译期确定的，无法在一个位置同时存放多种类型
    #[derive(Debug, Clone, Copy)]
    pub enum JumpStat {
        /// 从未进行过跳跃，因而能够获得一些来自设计师的宽容
        NeverJump,
        /// 大跳中，意味着在持续克服重力（违背正常物理规律）
        ///
        /// 期间上升速度恒定或者重力影响小，结束后正常受重力影响
        HigherJumping,
        /// 完成跳跃，普通跳跃的速度改变是一瞬间的，因此仅意味着该状态正常受重力影响，可能仍因为惯性处于上升阶段
        Jumped,
    }

    impl JumpStat {
        fn new() -> Self {
            Self::NeverJump
        }

        fn init(&mut self) {
            *self = Self::NeverJump
        }

        fn higher_jump(&mut self) {
            *self = Self::HigherJumping
        }

        fn jumped(&mut self) {
            *self = Self::Jumped
        }
    }

    /// 跳跃行为的辅助工具
    ///
    /// 包含状态管理、郊狼时间、大跳计时
    pub struct JumpBehaviourHelper {
        /// 跳跃状态，一个简单的有限状态机
        jump_stat: JumpStat,
        /// 郊狼时间，跳跃操作体验优化，给予容错时间
        coyote_time: TickTimerFinite,
        /// 大跳
        higher_jump: TickTimerFinite,
    }

    impl JumpBehaviourHelper {
        /// 必须先 [`Self::init`] 才能使用
        ///
        /// 郊狼时间参考值 0.1s
        pub fn new(coyote_time_limit: f64, higher_jump_duration: f64) -> Self {
            Self {
                jump_stat: JumpStat::new(),
                coyote_time: TickTimerFinite::new(coyote_time_limit),
                higher_jump: TickTimerFinite::new(higher_jump_duration),
            }
        }

        /// 行为逻辑中，每次进入空中状态时，检测是否存在向上速度
        ///
        /// 若无向上速度则认为是非跳跃进入的此状态，初始化为【未跳跃状态】
        ///
        /// 若脚部触碰到可跳跃媒介，通用初始化为【未跳跃状态】
        pub fn init(&mut self) {
            self.jump_stat.init();
            // 【未跳跃状态】自动重启郊狼时间
            self.coyote_time.restart();
            // 【未跳跃状态】自动结束大跳时间
            self.higher_jump.finish();
        }

        /// 若跳跃进入空中状态，则进入【大跳状态】
        ///
        /// 每一次新的跳跃行为都默认进入【大跳状态】
        pub fn higher_jump(&mut self) {
            self.jump_stat.higher_jump();
            // 进入【大跳状态】自动结束郊狼时间
            self.coyote_time.finish();
            // 进入【大跳状态】自动重启大跳时间
            self.higher_jump.restart();
        }

        /// 若结束【大跳状态】，则【跳跃完成】
        pub fn complete_a_jump(&mut self) {
            self.jump_stat.jumped();
            // 【跳跃完成】自动结束郊狼时间
            self.coyote_time.finish();
            // 【跳跃完成】自动结束大跳时间
            self.higher_jump.finish();
        }

        /// 根据当前状态来判断指令能否生效
        pub fn get_stat(&self) -> JumpStat {
            self.jump_stat
        }

        /// 是否在郊狼时间内
        ///
        /// 若业务侧判断能够直接进行跳跃，则无需调用本方法，本方法之后才应判断二段跳
        pub fn can_coyote_jump(&self) -> bool {
            // 计时与状态始终保持自洽 无需判断状态
            !self.coyote_time.is_finished()
        }

        /// 是否在大跳时间内
        pub fn in_higher_jumping(&self) -> bool {
            // 计时与状态始终保持自洽 无需判断状态
            !self.higher_jump.is_finished()
        }
    }

    impl TickTimer for JumpBehaviourHelper {
        fn tick(&mut self, delta: f64) {
            self.coyote_time.tick(delta);
            self.higher_jump.tick(delta);
        }
    }
}

/// 地面一般行为 todo
mod on_land_behaviour {

}

/// 攻击行为举例
///
/// - 命中派生，攻击命中敌人增加 tag
/// - 自动反击，受击自动增加 tag ，同时需要对伤害系统做 hook
mod attack_example_behaviour {}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_in_air_jump() {
        todo!()
    }
}

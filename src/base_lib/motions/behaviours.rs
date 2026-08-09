//! 控制一个动作内部具体的行为
//!
//! 这里仅作样例或提供辅助工具，具体应该在业务侧根据策划需求去实现
//!
//! ## 架构设计
//!
//! action 和 behaviour 的层级如何设计？
//!
//! - 先假设 action 下面细分 behaviour ，通过动作切换器决定进行哪个动作，简单动作直接播放动画，复杂动作对应一个行为
//!
//! 提出问题：
//!
//! - 如果把 in_air 作为动作下的行为，那么空中攻击动作完成后重新进入 in_air 状态，会导致跳跃行为的相关参数重置，比如重置二段跳次数，这个不符合预期
//!
//! 决策解决方案：
//!
//! - （旧架构）用 behaviour 作为主状态，内部 action 覆盖 behaviour 的动画
//!   - 【缺点】behaviour 需要独立状态机控制状态切换，增加了复杂性
//!   - 【缺点】base behaviour 作为一个通用逻辑，需要作特殊处理，违反架构设计
//!   - 举例：空中攻击动作属于空中行为，地面攻击动作属于地面行为
//! - （新架构）保持 action-behaviour 的层级顺序，但令 behaviour 之间存在依赖关系，如空中攻击动作的行为依赖 in_air 和 base ，只有依赖切换时才执行退出逻辑
//!   - 【优点】借助 action 实现自动状态转换，架构统一
//!   - 【优点】兼容 base behaviour 公共逻辑，且可拓展
//!   - 举例：空中攻击动作依赖空中行为，因此在空中触发攻击不会导致空中行为退出
//!
//! 基本实现：
//!
//! - 动作切换器决定当前做哪个动作
//! - 简单动作直接对应一个动画数据，没有行为逻辑，直接触发动画
//! - 复杂动作额外关联一个行为逻辑，内部业务具体实现，更复杂的可分层子行为
//! - 行为不激活时不执行对应逻辑，行为可依赖其他行为来激活对应逻辑
//! - 切换行为前先切换依赖行为，依赖行为与上一行为一致的不执行退出重入逻辑
//! - 简单动作无行为时不会触发现有依赖行为的退出
//! - 进入逻辑先执行依赖行为，退出逻辑反之
//! - 预期的依赖行为不是很多，使用 list 记录 id 和上一行为进行对比
//! - 设计依赖行为是扁平化的，不递归，因此在行为初始化时对依赖行为进行下钻，检查依赖的依赖是否在依赖列表中
//! - 行为应该是纯函数，副作用通过返回值传递，比如修改动画或者物理等，在框架侧进行优先级聚合后统一实施作用
//!   - 聚合逻辑若要追求完美，可能会特别复杂
//!   - 可简化为字段覆盖，互斥的字段组合成一个类型，根据具体效果灵活变通

/// 抽象行为，定义特征/接口/虚类
///
/// 部分复杂的动作需要行为进行拓展，一个动作可对应零或一个行为，复杂行为可在内部细分
///
/// - on_enter
/// - on_exit
/// - tick 帧逻辑，一般在 _physics_process / FixedUpdate 中调用
pub mod abstract_behaviour {}

/// 基础行为，存放通用逻辑，无论什么行为都会去调用
///
/// 识别【客观条件】，自动更新【运动模式】 tag
///
/// - climb_wall
/// - in_air
/// - on_land
/// - under_water
pub mod base_behaviour {}

/// 依赖特定 tag 才能进入的高优先级动作行为，不会切换至其他状态，打开上帝模式在测试时用
pub mod god_behaviour {}

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
        pub fn is_higher_jumping(&self) -> bool {
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

/// 地面一般行为
///
/// - hard-landing 硬着陆眩晕效果通过动作系统实现
/// - 着陆受身动画，进入状态时，计算上一帧y轴速度差值，差值过大播放受身动画，行为内部优先级最高，仅为视觉效果
///   - 参考动画时长 0.1s
/// - 起跳动画，受到跳跃指令播放起跳动画，而后消费指令进行跳跃，落地允许直接跳跃、连点跳跃不重复播放起跳动画
///   - 快节奏/硬核 0.05 - 0.15 秒
///   - 中节奏/流畅 0.15 - 0.2  秒
///   - 慢节奏/蓄力 0.2  - 0.5  秒 （超过 0.25 秒会有严重的输入延迟）
///   - 起跳动画期间若离开平台，则通过郊狼时间实现跳跃 （因此必须真正起跳时才消费指令）
/// - 转身动画，速度很快时输入反方向移动指令，触发转身动画，转身动画播放时取消转向、结束时自动播放奔跑动画
/// - 根据是否移动切换站立和行走动画
pub mod on_land_behaviour {
    use crate::base_lib::cores::{
        tick_timer::TickTimerFinite,
        tiny_timer::{FlowingTimer, FlowingTimerReadonly, TickTimer},
    };

    /// 落地受身辅助，动画驱动
    pub struct LandingRoll {
        /// 当前正在播放落地受身动画
        anim_playing: bool,
    }

    /// 落地立即起跳辅助，计时器和动画驱动
    pub struct ReadyToJump {
        /// 参考 coyote_time ，落地的一段时间内【允许立即跳跃】，跳过起跳动画
        allow_immediate_jump: TickTimerFinite,
    }

    impl LandingRoll {
        pub fn new() -> Self {
            Self {
                anim_playing: false,
            }
        }

        /// 大落差、速度差超过阈值，需要落地受身
        pub fn init(&mut self, big_fall_to_land: bool) {
            // 只有大落差才能够开启标记
            self.anim_playing = big_fall_to_land;
        }

        /// 是否播放落地受身动画
        ///
        /// - landing_anim_finished - 当前正在播放受身动画、且动画播放结束
        pub fn should_play_anim(&mut self, landing_anim_finished: bool) -> bool {
            if landing_anim_finished {
                // 只有动画播放完成才能够关闭标记
                self.anim_playing = false;
            }
            self.anim_playing
        }
    }

    impl ReadyToJump {
        pub fn new(jump_immediately_time: f64) -> Self {
            Self {
                allow_immediate_jump: TickTimerFinite::new(jump_immediately_time),
            }
        }

        pub fn init(&mut self, fall_to_land: bool) {
            if fall_to_land {
                self.allow_immediate_jump.restart();
            } else {
                self.allow_immediate_jump.finish();
            }
        }

        /// 是否触发跳跃
        ///
        /// - ready_to_jump_anim_finished - 当前正在播放起跳动画、且动画播放结束
        pub fn jump_immediately(&self, ready_to_jump_anim_finished: bool) -> bool {
            ready_to_jump_anim_finished || !self.allow_immediate_jump.is_finished()
        }
    }

    impl TickTimer for ReadyToJump {
        fn tick(&mut self, delta: f64) {
            self.allow_immediate_jump.tick(delta);
        }
    }
}

/// 攻击行为举例
///
/// - 命中派生，攻击命中敌人增加 tag
/// - 自动反击，受击自动增加 tag ，同时需要对伤害系统做 hook
pub mod attack_example_behaviour {}

#[cfg(test)]
mod tests {
    use crate::base_lib::{
        cores::tiny_timer::TickTimer,
        motions::behaviours::{
            in_air_behaviour::JumpBehaviourHelper,
            on_land_behaviour::{LandingRoll, ReadyToJump},
        },
    };

    #[test]
    fn test_in_air_jump() {
        let mut jump_behaviour_helper = JumpBehaviourHelper::new(0.1, 0.4);

        // 计时尽量长，先业务后 tick
        jump_behaviour_helper.init();
        assert!(jump_behaviour_helper.can_coyote_jump()); // 未跳跃，允许郊狼跳跃
        assert!(!jump_behaviour_helper.is_higher_jumping()); // 未跳跃，不在大跳时间内
        jump_behaviour_helper.tick(0.3);

        assert!(!jump_behaviour_helper.can_coyote_jump()); // 郊狼时间结束
        assert!(!jump_behaviour_helper.is_higher_jumping()); // 未跳跃，不在大跳时间内
        jump_behaviour_helper.higher_jump(); // 二段跳强制跳跃
        assert!(!jump_behaviour_helper.can_coyote_jump()); // 大跳，无法触发郊狼跳跃
        assert!(jump_behaviour_helper.is_higher_jumping()); // 大跳进行中
        jump_behaviour_helper.tick(1.0);

        assert!(!jump_behaviour_helper.can_coyote_jump());
        assert!(!jump_behaviour_helper.is_higher_jumping()); // 大跳结束
        jump_behaviour_helper.higher_jump(); // 又一次二段跳
        jump_behaviour_helper.tick(0.2);

        assert!(!jump_behaviour_helper.can_coyote_jump()); // 大跳，无法触发郊狼跳跃
        assert!(jump_behaviour_helper.is_higher_jumping()); // 大跳仍在进行中
        jump_behaviour_helper.complete_a_jump(); // 主动结束大跳
        assert!(!jump_behaviour_helper.can_coyote_jump());
        assert!(!jump_behaviour_helper.is_higher_jumping()); // 大跳结束
        jump_behaviour_helper.tick(1.0);
    }

    #[test]
    fn test_landing_roll() {
        let mut landing_roll = LandingRoll::new();

        // enter with big_fall_to_land
        let big_fall_to_land = true;
        landing_roll.init(big_fall_to_land);

        // tick first
        let playing_landing_anim = false;
        let landing_anim_finished = false;
        let should_play =
            landing_roll.should_play_anim(playing_landing_anim && landing_anim_finished);
        assert!(should_play);
        // do play landing anim

        // tick palying
        let playing_landing_anim = true;
        let landing_anim_finished = false;
        let should_play =
            landing_roll.should_play_anim(playing_landing_anim && landing_anim_finished);
        assert!(should_play);
        // continue play landing anim

        // tick play finished
        let playing_landing_anim = true;
        let landing_anim_finished = true;
        let should_play =
            landing_roll.should_play_anim(playing_landing_anim && landing_anim_finished);
        assert!(!should_play);
        // do play other anim

        // tick play other anim
        let playing_landing_anim = false;
        let landing_anim_finished = false;
        let should_play =
            landing_roll.should_play_anim(playing_landing_anim && landing_anim_finished);
        assert!(!should_play);
        // do nothing

        // ===========================

        // enter without big_fall_to_land
        let big_fall_to_land = false;
        landing_roll.init(big_fall_to_land);

        // tick
        let playing_landing_anim = false;
        let landing_anim_finished = false;
        let should_play =
            landing_roll.should_play_anim(playing_landing_anim && landing_anim_finished);
        assert!(!should_play);
        // do nothing
    }

    #[test]
    fn test_ready_to_jump() {
        let mut ready_to_jump = ReadyToJump::new(0.2);

        // 计时尽量长，先业务后 tick

        // enter with fall_to_land
        let fall_to_land = true;
        ready_to_jump.init(fall_to_land);

        // tick
        let playing_ready_to_jump_anim = false;
        let ready_to_jump_anim_finished = false;
        let can_jump = ready_to_jump
            .jump_immediately(playing_ready_to_jump_anim && ready_to_jump_anim_finished);
        assert!(can_jump);
        // but not jump
        ready_to_jump.tick(0.1);

        // tick
        let playing_ready_to_jump_anim = false;
        let ready_to_jump_anim_finished = false;
        let can_jump = ready_to_jump
            .jump_immediately(playing_ready_to_jump_anim && ready_to_jump_anim_finished);
        assert!(can_jump);
        // but not jump, ready_to_jump 时间窗过期
        ready_to_jump.tick(0.1);

        // tick
        let playing_ready_to_jump_anim = false;
        let ready_to_jump_anim_finished = false;
        let can_jump = ready_to_jump
            .jump_immediately(playing_ready_to_jump_anim && ready_to_jump_anim_finished);
        assert!(!can_jump);
        // want jump, play ready_to_jump anim
        ready_to_jump.tick(0.1);

        // tick
        let playing_ready_to_jump_anim = true;
        let ready_to_jump_anim_finished = false;
        let can_jump = ready_to_jump
            .jump_immediately(playing_ready_to_jump_anim && ready_to_jump_anim_finished);
        assert!(!can_jump);
        // playing ready_to_jump anim
        ready_to_jump.tick(0.1);

        // tick
        let playing_ready_to_jump_anim = true;
        let ready_to_jump_anim_finished = true;
        let can_jump = ready_to_jump
            .jump_immediately(playing_ready_to_jump_anim && ready_to_jump_anim_finished);
        assert!(can_jump);
        // ready_to_jump anim finished, do jump
        ready_to_jump.tick(0.1);

        // exit on_land behavior
    }
}

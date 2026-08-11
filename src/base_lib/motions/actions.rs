//! 控制角色动作切换
//!
//! 使用 tag 定制切换逻辑，用于代替状态机转换
//!
//! 传统状态机存在以下缺点
//! - 可维护性差，单个状态可能存在扇入扇出的情况，此时修改容易引发蝴蝶效应
//! - 可视化差，复杂网状结构几乎不可读
//! - 虽然可用分层状态机优化，但是统一层级的动作过多时也会有一样的问题
//!
//! 优先级动作切换器（目前的方案）
//! - 基于标签实现，支持复杂切换逻辑，但是把复杂度转移到了优先级上面，因此需要设计好优先级级别
//! - 可视化差，需要自己实现预览视图，同时预览视图只能看到标签条件跳转到动作，无法直接看到动作的跳转逻辑
//!
//! 动作切换系统参考自 <https://github.com/kierstone/ACT-Game-Action-System/tree/main>
//! - 原文 <https://mp.weixin.qq.com/s?__biz=MzA3NjQzMzYxMw==&mid=2650635859&idx=1&sn=ba171829af2fc461f5e5be9dcafc804e&chksm=87688ff1b01f06e77d42952eb6207feb736781798c7bdaaeb6df60f40986d6a452e8824ed94c&scene=23&srcid=1025GxLrb9dTlBpNgWSbaQiN>
//! - 讲解了以“动作帧”为单位，如何实现动作选择器，以支持 ACT 游戏开发（非 ARPG 游戏）
//! - 这里参考了 UE GAS 框架里面的 GameplayTag ，通过 Tag 来解耦框架实现与合业务逻辑（不在强制绑定动作取消标记和、取消条件、自然结束后的动作等等）

use rustc_hash::FxHashMap;

use crate::base_lib::cores::{
    design_patterns::Union,
    timers::{
        pause_prefab::PausePrefab,
        tick_timer::TickTimer,
        tiny_timer::{Tickable, TimerPauseControl, TimerPauseView, TimerView},
    },
    tiny_tags::{TinyTag, TinyTagContainer},
    unify_types::FixedName,
};

/// 动作数据
pub struct ActionData<PureTag: FixedName> {
    // 基础信息
    // ===========================
    /// id 应在工程打包时确定
    id: i64,
    /// 优先级，选择动作时用于决策，值大的优先，优先级相同时根据注册顺序决定，后注册的优先
    priority: u16,
    /// 注册顺序，该值在注册的时候自动赋值
    order: usize,

    // 进入条件
    // ===========================
    /// 进入条件
    ///
    /// 复杂条件可拆分为 (前置动作集, 细分子状态, 触发事件)
    ///
    /// 基础条件建议设置为 Not(xxx) 而不要 Always ，以保留强制剔除动作的灵活性，但至少要有一个 Always 条件
    enter_confition: TinyTag<PureTag>,

    // 初始化逻辑
    // ===========================
    /// 状态 tag 生命周期与当前动作一致
    state_tags: Vec<PureTag>,
}

impl<PureTag: FixedName> ActionData<PureTag> {
    /// 判断优先级，若优先级相同则根据注册顺序判断
    fn priority_over(&self, other: &Self) -> bool {
        self.priority > other.priority
            || self.priority == other.priority && self.order > other.order
    }

    /// 判断优先级，若优先级相同则根据注册顺序判断，若为空则始终优先
    fn priority_over_opt(&self, other: Option<&Self>) -> bool {
        match other {
            Some(other) => self.priority_over(other),
            None => true,
        }
    }
}

/// tag 集合
struct ActionTagContainer<PureTag: FixedName>(FxHashMap<PureTag, Option<TickTimer>>);

impl<PureTag: FixedName> TinyTagContainer for ActionTagContainer<PureTag> {
    type Element = PureTag;

    fn check_condition(&self, pure_tag: &Self::Element) -> bool {
        self.0.contains_key(pure_tag)
    }
}

impl<PureTag: FixedName> Tickable for ActionTagContainer<PureTag> {
    fn tick(&mut self, delta: f64) {
        // 清理过期的 tag
        self.0.retain(|_k, v| {
            if let Some(timer) = v {
                // 计时生命
                timer.tick(delta);
                !timer.is_completed()
            } else {
                // 无限生命
                true
            }
        });
    }
}

/// 动作切换器
pub struct ActionSwitcher<PureTag: FixedName> {
    /// 动作数据库
    action_database: FxHashMap<i64, ActionData<PureTag>>,

    /// 通过 tag 控制动作切换，业务实现层可在通用模块（共同逻辑）和动作模块（独特逻辑）分别定制刷新 tag 逻辑
    current_tags: ActionTagContainer<PureTag>,

    /// 当前动作 id
    current_action_id: i64,

    /// 暂停 tag 计时
    pause_prefab: PausePrefab,
}

impl<PureTag: FixedName> ActionSwitcher<PureTag> {
    pub fn new(default_action: i64) -> Self {
        Self {
            action_database: FxHashMap::default(),
            current_tags: ActionTagContainer(FxHashMap::default()),
            current_action_id: default_action,
            pause_prefab: PausePrefab::default(),
        }
    }

    pub fn register_action(&mut self, mut action: ActionData<PureTag>) {
        // 无法注销，因此直接使用当前个数作为注册顺序
        action.order = self.action_database.len();
        self.action_database.insert(action.id, action);
    }

    /// 切换动作，返回当前动作
    pub fn switch_next_action(&mut self) -> i64 {
        // 候选动作
        let mut candidates: Option<&ActionData<PureTag>> = None;
        for action in self.action_database.values() {
            // 跳过自身 防止自己切换到自己导致修改 tag
            if action.id == self.current_action_id {
                continue;
            }

            // 优先级排序
            if action.priority_over_opt(candidates) {
                // 条件判断
                if action.enter_confition.check_condition(&self.current_tags) {
                    candidates = Some(action);
                }
            }
        }

        if let Some(candidates) = candidates {
            return self.do_switch_action(candidates.id);
        } else {
            // 空不切换
            return self.current_action_id;
        }
    }

    /// 切换动作，保证切换的动作始终存在，返回当前动作
    fn do_switch_action(&mut self, action_id: i64) -> i64 {
        if let Some(new_action) = self.action_database.get(&action_id) {
            if let Some(old_action) = self.action_database.get(&self.current_action_id) {
                for tag in &old_action.state_tags {
                    self.current_tags.0.remove(&tag);
                }
            }
            self.current_action_id = action_id;
            for tag in &new_action.state_tags {
                // 状态 tag 都是无限生命周期 采用手动管理方式
                self.current_tags.0.insert(tag.clone(), None);
            }
        }
        return self.current_action_id;
    }

    /// 事件触发增加有时效性的 tag
    pub fn upsert_timer_tag(&mut self, tag: PureTag, timer: TickTimer) {
        self.current_tags.0.insert(tag, Some(timer));
    }

    /// 获取当前所有的 tag ，一般用于调试
    pub fn get_current_tags(&self) -> Vec<PureTag> {
        self.current_tags.0.keys().cloned().collect()
    }
}

// region: impl freezable tick

impl<PureTag: FixedName> Tickable for ActionSwitcher<PureTag> {
    fn tick(&mut self, delta: f64) {
        Union::new(&self.pause_prefab, &mut self.current_tags).tick(delta);
    }
}

impl<PureTag: FixedName> TimerPauseView for ActionSwitcher<PureTag> {
    fn is_paused(&self) -> bool {
        Union::new(&self.pause_prefab, &self.current_tags).is_paused()
    }
}

impl<PureTag: FixedName> TimerPauseControl for ActionSwitcher<PureTag> {
    fn pause(&mut self) {
        Union::new(&mut self.pause_prefab, &self.current_tags).pause();
    }

    fn resume(&mut self) {
        Union::new(&mut self.pause_prefab, &self.current_tags).resume();
    }
}

// endregion

use rustc_hash::FxHashMap;

use crate::base_lib::cores::{
    design_patterns::WithContext,
    tick_timer::TickTimerFinite,
    tiny_tags::{TinyTag, TinyTagContainer},
    tiny_timer::{
        FlowingTimerReadonly, FreezableTimer, FreezableTimerReadonly, TickTimer,
        freezable_tick::FreezableTickTag,
    },
    unify_types::FixedName,
};

/// 动作数据
pub struct ActionData<PureTag: FixedName> {
    // 基础信息
    // ===========================
    /// id 应在工程打包时确定
    id: i64,
    /// 优先级，选择动作时用于决策，值大的优先，优先级相同时随机选取（根据底层 HashMap 的排序来）
    priority: u16,

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

/// tag 集合
struct ActionTagContainer<PureTag: FixedName>(FxHashMap<PureTag, Option<TickTimerFinite>>);

impl<PureTag: FixedName> TinyTagContainer for ActionTagContainer<PureTag> {
    type Element = PureTag;

    fn check_condition(&self, pure_tag: &Self::Element) -> bool {
        self.0.contains_key(pure_tag)
    }
}

impl<PureTag: FixedName> TickTimer for ActionTagContainer<PureTag> {
    fn tick(&mut self, delta: f64) {
        // 清理过期的 tag
        self.0.retain(|_k, v| {
            if let Some(timer) = v {
                // 计时生命
                timer.tick(delta);
                !timer.is_finished()
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
    freezable_tick_tag: FreezableTickTag,
}

impl<PureTag: FixedName> ActionSwitcher<PureTag> {
    pub fn new(default_action: i64) -> Self {
        Self {
            action_database: FxHashMap::default(),
            current_tags: ActionTagContainer(FxHashMap::default()),
            current_action_id: default_action,
            freezable_tick_tag: FreezableTickTag::default(),
        }
    }

    pub fn register_action(&mut self, action: ActionData<PureTag>) {
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

            // 优先级排序 选择大的
            if candidates.is_none() || candidates.is_some_and(|c| c.priority < action.priority) {
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
    pub fn upsert_timer_tag(&mut self, tag: PureTag, timer: TickTimerFinite) {
        self.current_tags.0.insert(tag, Some(timer));
    }
}

// region: impl freezable tick

impl<PureTag: FixedName> TickTimer for ActionSwitcher<PureTag> {
    fn tick(&mut self, delta: f64) {
        self.freezable_tick_tag
            .with_ctx_mut(&mut self.current_tags)
            .tick(delta);
    }
}

impl<PureTag: FixedName> FreezableTimerReadonly for ActionSwitcher<PureTag> {
    fn is_frozen(&self) -> bool {
        self.freezable_tick_tag
            .with_ctx(&self.current_tags)
            .is_frozen()
    }
}

impl<PureTag: FixedName> FreezableTimer for ActionSwitcher<PureTag> {
    fn freeze(&mut self) {
        self.freezable_tick_tag
            .with_ctx_mut(&mut self.current_tags)
            .freeze();
    }

    fn resume(&mut self) {
        self.freezable_tick_tag
            .with_ctx_mut(&mut self.current_tags)
            .resume();
    }
}

// endregion

use rustc_hash::{FxHashMap, FxHashSet};

use crate::base_lib::cores::{
    tiny_tags::{TinyTag, TinyTagContainer},
    unify_types::FixedName,
};

/// 动作数据
pub struct ActionData<PureTag: FixedName> {
    // 基础信息
    // ===========================
    /// id 0 表示空
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
    /// 初始化时默认拥有 tag
    init_tags: Vec<PureTag>,
    /// 初始化时是否清空 tag
    clear_tags: bool,
}

/// tag 集合
pub struct ActionTagContainer<PureTag: FixedName>(FxHashSet<PureTag>);

impl<PureTag: FixedName> TinyTagContainer for ActionTagContainer<PureTag> {
    type Element = PureTag;

    fn check_condition(&self, pure_tag: &Self::Element) -> bool {
        self.0.contains(pure_tag)
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
}

impl<PureTag: FixedName> ActionSwitcher<PureTag> {
    pub fn new(default_action: i64) -> Self {
        Self {
            action_database: FxHashMap::default(),
            current_tags: ActionTagContainer(FxHashSet::default()),
            current_action_id: default_action,
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
        if let Some(action) = self.action_database.get(&action_id) {
            self.current_action_id = action_id;
            if action.clear_tags {
                self.current_tags.0.clear();
            }
            for tag in &action.init_tags {
                self.current_tags.0.insert(tag.clone());
            }
        }
        return self.current_action_id;
    }
}

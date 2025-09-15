use std::collections::{HashMap, HashSet};

use crate::{
    cores::unify_type::FixedString,
    movements::movement::{ActionExitLogic, ActionTrigger, MovementMode},
};

/// 动作 纯数据 实现固定效果
#[derive(Clone, Debug)]
pub struct Action<S: FixedString, PhysicalEffect: Copy> {
    /// 动作名称
    pub action_name: S,

    /// 本动作的触发指令与信号
    pub trigger: Vec<ActionTrigger>,
    /// 兼容的运动状态
    pub movement_modes: HashSet<MovementMode>,
    /// 切换运动状态是否保持（若本身兼容则无需填写）
    pub movement_keep: HashSet<MovementMode>,

    /// 退出逻辑与下一个动作
    pub exit_logic_and_next_action: HashMap<ActionExitLogic, S>,
    /// 指令与信号触发的下一个动作
    pub trigger_to_next_action: HashMap<ActionTrigger, S>,

    /// 动作优先级
    pub action_priority: i64,
    /// 动作自定义覆盖关系 true能被其他覆盖 false不能
    pub action_switch_relation: HashMap<S, ActionCanCover>,

    /// 初始播放的动画名称
    pub anim_first: S,
    /// 动画结束后自动播放的下一个动画
    pub anim_next: HashMap<S, S>,

    /// 动画进行时的物理效果
    pub anim_physics: HashMap<S, PhysicalEffect>,
    /// 指令与信号响应时的物理效果
    pub trigger_physics: HashMap<ActionTrigger, PhysicalEffect>,
}

#[derive(Clone, Copy, Debug)]
pub struct ActionCanCover(pub bool);

impl From<bool> for ActionCanCover {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

// #[derive(Clone, Copy, Debug)]
// pub struct ActionShouldForceStop(pub bool);

// impl From<bool> for ActionShouldForceStop {
//     fn from(value: bool) -> Self {
//         Self(value)
//     }
// }

impl<S: FixedString, P: Copy> Action<S, P> {
    pub fn new_empty(action_name: S, anim_first: S) -> Self {
        // 兼容全部运动模式
        let movement_modes = MovementMode::gen_set();
        Self {
            action_name,
            trigger: Vec::new(),
            movement_modes,
            movement_keep: HashSet::new(), // 因为兼容全部所以不用额外设置保持
            exit_logic_and_next_action: HashMap::new(),
            trigger_to_next_action: HashMap::new(),
            action_priority: 0,
            action_switch_relation: HashMap::new(),
            anim_first,
            anim_next: HashMap::new(),
            anim_physics: HashMap::new(),
            trigger_physics: HashMap::new(),
        }
    }

    pub fn should_trigger_when_movement(&self, movement_mode: &MovementMode) -> bool {
        self.movement_modes.contains(movement_mode)
    }

    pub fn should_force_stop_by_movement(&self, movement_mode: &MovementMode) -> bool {
        if self.movement_modes.contains(movement_mode) {
            true
        } else if self.movement_keep.contains(movement_mode) {
            true
        } else {
            false
        }
    }

    /// return None if should not exit
    pub fn should_exit_to_next_action(&self, exit_logic: &ActionExitLogic) -> Option<&S> {
        self.exit_logic_and_next_action.get(exit_logic)
    }

    /// return None if should not trigger other action
    pub fn should_trigger_to_next_action(&self, trigger: &ActionTrigger) -> Option<&S> {
        self.trigger_to_next_action.get(trigger)
    }

    /// 本动作可以切换到另一个动作
    pub fn should_switch_other_action(&self, other: &Self) -> bool {
        if let Some(can_cover) = self.action_switch_relation.get(&other.action_name) {
            can_cover.0
        } else {
            other.action_priority >= self.action_priority
        }
    }

    pub fn first_anim(&self) -> &S {
        &self.anim_first
    }

    /// return None if has no next anim
    pub fn next_anim(&self, cur_anim: &S) -> Option<&S> {
        self.anim_next.get(cur_anim)
    }

    pub fn get_physics_effect_by_anim(&self, anim: &S) -> Option<P> {
        self.anim_physics.get(anim).copied()
    }

    pub fn get_physics_effect_by_trigger(&self, trigger: &ActionTrigger) -> Option<P> {
        self.trigger_physics.get(trigger).copied()
    }
}

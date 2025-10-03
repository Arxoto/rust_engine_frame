//! 动作系统的状态机实现

use std::collections::HashMap;

use crate::{
    cores::unify_type::FixedString,
    motion::{
        action_types::ActionExitLogic,
        movement_action_impl::{ActionMovementExitParam, MovementActionEvent},
        state_machine_types_impl::{FrameParam, MovementAction, PhyParam},
    },
};

/// 动作状态机
///
/// 动作与动作之间有关联，面向数据设计，对分层状态也有一定程度上的支持
#[derive(Default)]
pub struct ActionMachine<S, PhyEff>
where
    S: FixedString,
{
    pub(crate) actions: HashMap<S, MovementAction<S, PhyEff>>,
    pub(crate) current_action_name: S,
    pub(crate) current_anim_name: S,
    pub(crate) event_trigger_actions: HashMap<MovementActionEvent, Vec<S>>,
}

impl<S, PhyEff> ActionMachine<S, PhyEff>
where
    S: FixedString,
{
    fn get_action(&self, action_name: &S) -> Option<&MovementAction<S, PhyEff>> {
        self.actions.get(action_name)
    }

    fn get_current_action(&self) -> Option<&MovementAction<S, PhyEff>> {
        self.get_action(&self.current_action_name)
    }

    fn do_update_anim_first_time(&mut self) {
        if let Some(the_action) = self.get_current_action() {
            self.current_anim_name = the_action.first_anim().clone();
        }
    }

    fn do_update_action(&mut self, next_action_name: S) {
        self.current_action_name = next_action_name;
        self.do_update_anim_first_time();
    }

    /// 动作内置的触发映射中 尝试获取下一个动作
    fn fetch_next_action_name_by_event_local(&self, e: &MovementActionEvent) -> Option<S> {
        let Some(current_action) = self.get_current_action() else {
            return None;
        };
        current_action
            .fetch_next_action_by_trigger(e)
            .map(|next_action_name| next_action_name.clone())
    }

    /// 全局触发映射中 尝试获取下一个动作
    fn fetch_next_action_name_by_event_global(&self, e: &MovementActionEvent) -> Option<S> {
        let Some(actions) = self.event_trigger_actions.get(e) else {
            return None;
        };

        let Some(current_action) = self.get_current_action() else {
            // just first
            return actions
                .get(0)
                .map(|next_action_name| next_action_name.clone());
        };

        for next_action_name in actions.iter() {
            let Some(next_action) = self.get_action(next_action_name) else {
                continue;
            };
            if current_action.can_switch_other_action(next_action) {
                return Some(next_action_name.clone());
            }
        }
        return None;
    }

    fn fetch_next_action_name_by_event(&self, e: &MovementActionEvent) -> Option<S> {
        self.fetch_next_action_name_by_event_local(e)
            .or_else(|| self.fetch_next_action_name_by_event_global(e))
    }

    fn fetch_next_action_name_by_tick(&self, p: &ActionMovementExitParam) -> Option<S> {
        let Some(the_action) = self.get_current_action() else {
            return None;
        };
        for (exit_logic, next_action_name) in the_action.tick_to_next_action.iter() {
            if exit_logic.should_exit(p) {
                return Some(next_action_name.clone());
            }
        }
        return None;
    }

    /// 事件触发的状态更新
    pub(crate) fn update_action_by_event(&mut self, e: &MovementActionEvent) {
        if let Some(next_action_name) = self.fetch_next_action_name_by_event(e) {
            self.do_update_action(next_action_name);
        }
    }

    /// 每帧进行状态更新
    pub(crate) fn update_action_by_tick(&mut self, p: &ActionMovementExitParam) {
        if let Some(next_action_name) = self.fetch_next_action_name_by_tick(p) {
            self.do_update_action(next_action_name);
        }
    }

    /// 渲染帧执行 返回当前帧的动画名称
    pub(crate) fn tick_frame(&mut self, p: &FrameParam<S>) -> &S {
        if p.anim_name == self.current_anim_name && p.anim_finished {
            // new anim
            if let Some(action) = self.get_current_action() {
                if let Some(next_anim_name) = action.next_anim(&self.current_anim_name) {
                    self.current_anim_name = next_anim_name.clone();
                }
            }
        }
        &self.current_anim_name
    }

    /// 物理帧执行 返回物理效果
    pub(crate) fn tick_physics(&mut self, p: &PhyParam<S>) -> Option<&PhyEff> {
        if let Some(action) = self.get_current_action() {
            action.get_phy_eff_by_anim(&p.anim_name)
        } else {
            None
        }
    }

    /// 初始化默认动作
    pub fn init_action(&mut self, a: &MovementAction<S, PhyEff>) {
        self.current_action_name = a.action_name.clone();
        self.current_anim_name = a.first_anim().clone();
    }

    /// 初始化时新增
    pub fn add_action(&mut self, a: MovementAction<S, PhyEff>) {
        // set trigger
        for event in a.trigger.iter() {
            if let Some(actions) = self.event_trigger_actions.get_mut(event) {
                actions.push(a.action_name.clone());
            } else {
                self.event_trigger_actions
                    .insert(event.clone(), vec![a.action_name.clone()]);
            }
        }

        // add to action map
        self.actions.insert(a.action_name.clone(), a);
    }
}

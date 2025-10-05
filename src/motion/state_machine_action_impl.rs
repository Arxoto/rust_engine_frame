//! 动作系统的状态机实现

use std::collections::HashMap;

use crate::{
    cores::unify_type::FixedString,
    motion::{
        action_types::ActionExitLogic,
        movement_action_impl::MovementActionEvent,
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

    fn fetch_next_action_name_by_tick(&self, exit_param: &FrameParam<S>) -> Option<S> {
        let Some(the_action) = self.get_current_action() else {
            return None;
        };
        for (exit_logic, next_action_name) in the_action.tick_to_next_action.iter() {
            if exit_logic.should_exit(exit_param) {
                return Some(next_action_name.clone());
            }
        }
        return None;
    }

    /// 事件触发的状态更新
    pub(crate) fn update_action_by_event(&mut self, e: &MovementActionEvent) -> bool {
        if let Some(next_action_name) = self.fetch_next_action_name_by_event(e) {
            self.do_update_action(next_action_name);
            return true;
        }
        return false;
    }

    /// 每帧进行状态更新
    pub(crate) fn update_action_by_tick(&mut self, exit_param: &FrameParam<S>) -> bool {
        if let Some(next_action_name) = self.fetch_next_action_name_by_tick(exit_param) {
            self.do_update_action(next_action_name);
            return true;
        }
        return false;
    }

    /// 渲染帧执行 使用 [`ActionMachine::get_frame_eff`] 获取结果
    ///
    /// 侧重处理 由于状态转换和帧处理通常一起调用 所以将帧处理的结果独立出来 支持两者的自定义顺序
    pub(crate) fn process_frame(&mut self, frame_param: &FrameParam<S>) {
        // 若出现动画名称不对应的情况 说明外部没有遵从动作框架的逻辑 （如动作框架处于缺省状态时，使用行为框架进行覆盖）
        if frame_param.anim_finished && frame_param.anim_name == self.current_anim_name {
            // update anim
            // 1、注意这里的结果 None 在正常执行时不允许 要求必须在退出条件里进行动画的结束判断
            //     但这里无视也没有什么大问题 因为会导致视觉上动画卡住 因此能立即发现
            // 2、同时有另一个问题，由于 update 方法与 tick 方法相互独立（设计上是为了将信号更新和每帧执行解耦合）
            //     这样就导致了执行 tick 的时候无法感知道 update 是否改变了动作（同时为了一致也不应把 tick 和 update_tick 强行合并）
            //     当上一个动作满足动作完成的退出条件时，先进行了动作的切换，而后由于动画结束标志并未修改
            //     若下一个动作的第一个动画恰好与上一个最后一个动画相同时，会直接跳过该动画
            //     这个问题暂且当作特性处理，感觉存在会比修复更好
            if let Some(action) = self.get_current_action() {
                if let Some(next_anim_name) = action.next_anim(&self.current_anim_name) {
                    self.current_anim_name = next_anim_name.clone();
                }
            }
        }
    }

    /// 返回当前帧的动画名称
    ///
    /// 动作侧重数据，返回值一般认为是固定的，所以仅返回引用
    pub(crate) fn get_frame_eff(&self) -> &S {
        &self.current_anim_name
    }

    /// 物理帧执行 返回物理效果
    ///
    /// 动作侧重数据，返回值一般认为是固定的，所以仅返回引用
    pub(crate) fn tick_physics(&mut self, p: &PhyParam<S>) -> Option<&PhyEff> {
        if let Some(action) = self.get_current_action() {
            action.get_phy_eff_by_anim(&p.anim_name)
        } else {
            None
        }
    }

    /// 初始化默认动作
    pub fn init_action(&mut self, action_name: &S) {
        if let Some(_) = self.get_action(action_name) {
            self.do_update_action(action_name.clone());
        }
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

#[cfg(test)]
mod unit_tests {
    use crate::motion::{
        action::Action,
        action_impl::{ActionBaseEvent, ActionBaseExitLogic},
        movement_action_impl::MovementActionExitLogic,
        movement_impl::MovementMode,
    };

    use super::*;

    #[test]
    fn add_and_init_action() {
        let mut action_machine: ActionMachine<&'static str, ()> = ActionMachine::default();

        // =========
        // first action
        // =========

        let mut action: MovementAction<&'static str, ()> =
            Action::new_empty("defence_action", "defence_anim");
        // 对所有运动模式进行匹配【防御指令】
        action
            .trigger
            .append(&mut MovementActionEvent::new_each_movement(
                ActionBaseEvent::DefenceInstruction,
            ));

        action_machine.add_action(action);

        // test event trigger
        for ele in MovementMode::each_mode() {
            let event = MovementActionEvent::new(ActionBaseEvent::DefenceInstruction, *ele);
            let action_names = action_machine.event_trigger_actions.get(&event).unwrap();
            assert_eq!(action_names, &vec!["defence_action"]);
        }
        // test action map
        let the_action = action_machine.get_action(&"defence_action").unwrap();
        assert_eq!(the_action.anim_first, "defence_anim");

        // =========
        // second action
        // =========

        let mut action: MovementAction<&'static str, ()> =
            Action::new_empty("defence_action_2", "defence_anim_2");
        // 仅地面状态下的【受击信号】
        action.trigger.push(MovementActionEvent::new(
            ActionBaseEvent::DefenceInstruction,
            MovementMode::OnFloor,
        ));

        action_machine.add_action(action);

        // test event trigger
        let event =
            MovementActionEvent::new(ActionBaseEvent::DefenceInstruction, MovementMode::OnFloor);
        let action_names = action_machine.event_trigger_actions.get(&event).unwrap();
        assert_eq!(action_names, &vec!["defence_action", "defence_action_2"]);

        // test action map
        let the_action = action_machine.get_action(&"defence_action_2").unwrap();
        assert_eq!(the_action.anim_first, "defence_anim_2");

        // =========
        // test init
        // =========

        assert_eq!(action_machine.current_action_name, "");
        assert_eq!(action_machine.current_anim_name, "");

        action_machine.init_action(&"defence_action");
        assert_eq!(action_machine.current_action_name, "defence_action");
        assert_eq!(action_machine.current_anim_name, "defence_anim");

        // =========
        // test update
        // =========

        action_machine.do_update_action(&"defence_action_2");
        assert_eq!(action_machine.current_action_name, "defence_action_2");
        assert_eq!(action_machine.current_anim_name, "defence_anim_2");
    }

    #[test]
    fn update_by_event_global_none() {
        // update global (current is none)
        let mut action_machine: ActionMachine<&'static str, ()> = ActionMachine::default();

        action_machine.add_action(Action {
            trigger: vec![MovementActionEvent::new(
                ActionBaseEvent::AttackInstruction,
                MovementMode::OnFloor,
            )],
            ..Action::new_empty("1", "anim_first")
        });
        action_machine.add_action(Action {
            trigger: vec![MovementActionEvent::new(
                ActionBaseEvent::AttackInstruction,
                MovementMode::OnFloor,
            )],
            ..Action::new_empty("2", "anim_first")
        });

        assert_eq!(action_machine.current_action_name, "");
        action_machine.update_action_by_event(&MovementActionEvent {
            event: ActionBaseEvent::AttackInstruction,
            movement: MovementMode::OnFloor,
        });
        assert_eq!(action_machine.current_action_name, "1");
    }

    #[test]
    fn update_by_event_global_some() {
        // update global (current is some, second action will be updated)
        let mut action_machine: ActionMachine<&'static str, ()> = ActionMachine::default();

        action_machine.add_action(Action {
            action_priority: 1,
            ..Action::new_empty("0", "anim_first")
        });
        action_machine.add_action(Action {
            trigger: vec![MovementActionEvent::new(
                ActionBaseEvent::AttackInstruction,
                MovementMode::OnFloor,
            )],
            action_priority: 0,
            ..Action::new_empty("1", "anim_first")
        });
        action_machine.add_action(Action {
            trigger: vec![MovementActionEvent::new(
                ActionBaseEvent::AttackInstruction,
                MovementMode::OnFloor,
            )],
            action_priority: 1,
            action_switch_relation: HashMap::from([("1", true)]),
            ..Action::new_empty("2", "anim_first")
        });

        action_machine.init_action(&"0");
        assert_eq!(action_machine.current_action_name, "0");

        action_machine.update_action_by_event(&MovementActionEvent {
            event: ActionBaseEvent::AttackInstruction,
            movement: MovementMode::OnFloor,
        });
        assert_eq!(action_machine.current_action_name, "2"); // because action_priority

        action_machine.update_action_by_event(&MovementActionEvent {
            event: ActionBaseEvent::AttackInstruction,
            movement: MovementMode::OnFloor,
        });
        assert_eq!(action_machine.current_action_name, "1"); // because action_switch_relation
    }

    #[test]
    fn update_by_event_local() {
        // update local
        let mut action_machine: ActionMachine<&'static str, ()> = ActionMachine::default();

        action_machine.add_action(Action {
            action_priority: 1,
            trigger_to_next_action: HashMap::from([(
                MovementActionEvent::new(ActionBaseEvent::AttackInstruction, MovementMode::OnFloor),
                "1",
            )]),
            ..Action::new_empty("0", "anim_first")
        });
        action_machine.add_action(Action {
            trigger: vec![MovementActionEvent::new(
                ActionBaseEvent::AttackInstruction,
                MovementMode::OnFloor,
            )],
            action_priority: 0,
            ..Action::new_empty("1", "anim_first")
        });
        action_machine.add_action(Action {
            trigger: vec![MovementActionEvent::new(
                ActionBaseEvent::AttackInstruction,
                MovementMode::OnFloor,
            )],
            action_priority: 1,
            action_switch_relation: HashMap::from([("1", true)]),
            ..Action::new_empty("2", "anim_first")
        });

        action_machine.init_action(&"0");
        assert_eq!(action_machine.current_action_name, "0");

        action_machine.update_action_by_event(&MovementActionEvent {
            event: ActionBaseEvent::AttackInstruction,
            movement: MovementMode::OnFloor,
        });
        assert_eq!(action_machine.current_action_name, "1"); // because trigger_to_next_action
    }

    #[test]
    fn update_by_tick() {
        let mut action_machine: ActionMachine<&'static str, ()> = ActionMachine::default();

        action_machine.add_action(Action {
            tick_to_next_action: vec![(
                MovementActionExitLogic::ExitLogic(ActionBaseExitLogic::AnimFinished("anim_first")),
                "1",
            )],
            ..Action::new_empty("0", "anim_first")
        });
        action_machine.add_action(Action {
            ..Action::new_empty("1", "anim_second")
        });

        action_machine.init_action(&"0");
        assert_eq!(action_machine.current_action_name, "0");
        assert_eq!(action_machine.current_anim_name, "anim_first");

        action_machine.update_action_by_tick(&FrameParam {
            anim_finished: true,
            anim_name: "anim_first",
            ..Default::default()
        });
        assert_eq!(action_machine.current_action_name, "1");
        assert_eq!(action_machine.current_anim_name, "anim_second");
    }

    #[test]
    fn process_frame() {
        let mut action_machine: ActionMachine<&'static str, ()> = ActionMachine::default();

        // 一系列动作 环状结构
        action_machine.add_action(Action {
            anim_next: HashMap::from([("0", "1"), ("1", "2"), ("2", "1")]),
            ..Action::new_empty("action_name", "0")
        });
        action_machine.init_action(&"action_name");

        // 模拟异常情况（动作系统处于缺省状态时用行为系统覆盖，这种情况对于动作系统自己来说是异常情况）
        action_machine.process_frame(&FrameParam {
            anim_finished: true,
            anim_name: "none",
            ..Default::default()
        });
        // 异常情况不做改变
        assert_eq!(action_machine.get_frame_eff(), &"0");

        // 动作未完成
        action_machine.process_frame(&FrameParam {
            anim_finished: false,
            anim_name: "0",
            ..Default::default()
        });
        assert_eq!(action_machine.get_frame_eff(), &"0");

        // 动作 0 -> 1
        action_machine.process_frame(&FrameParam {
            anim_finished: true,
            anim_name: "0",
            ..Default::default()
        });
        assert_eq!(action_machine.get_frame_eff(), &"1");

        // 动作 1 -> 2
        action_machine.process_frame(&FrameParam {
            anim_finished: true,
            anim_name: "1",
            ..Default::default()
        });
        assert_eq!(action_machine.get_frame_eff(), &"2");

        // 动作 2 -> 1
        action_machine.process_frame(&FrameParam {
            anim_finished: true,
            anim_name: "2",
            ..Default::default()
        });
        assert_eq!(action_machine.get_frame_eff(), &"1");

        // 动作 1 -> 2 循环
        action_machine.process_frame(&FrameParam {
            anim_finished: true,
            anim_name: "1",
            ..Default::default()
        });
        assert_eq!(action_machine.get_frame_eff(), &"2");
    }
}

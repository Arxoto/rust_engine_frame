//! 动作系统的动作数据结构

use std::collections::HashMap;

use crate::{
    cores::unify_type::FixedString,
    motions::abstracts::action_types::{ActionEvent, ActionExitLogic},
};

/// 动作 纯数据 实现固定效果
///
/// 若想实现高级的动画效果（如上下半身动画播放器分离），则外部自己实现组合逻辑
#[derive(Clone, Debug)]
pub struct Action<S, Event, ExitParam, ExitLogic, PhyEff>
where
    S: FixedString,
    Event: ActionEvent,
    ExitLogic: ActionExitLogic<ExitParam>,
{
    /// 动作名称
    pub(crate) action_name: S,

    /// 定义接收到何种事件能进入该动作
    pub(crate) event_enter: Vec<Event>,
    /// 定义接收到事件后切换到其他动作
    pub(crate) event_exit: HashMap<Event, S>,
    // todo 数据结构类型修改为 Vec
    /// 定义满足一定逻辑条件后切换到其他动作
    pub(crate) logic_exit: Vec<(ExitLogic, S)>,

    /// 动作优先级
    pub(crate) action_priority: i64,
    /// 动作自定义覆盖关系 true能被其他覆盖 false不能
    pub(crate) action_switch_relation: HashMap<S, bool>,

    /// 初始播放的动画名称
    pub(crate) anim_first: S,
    /// 动画结束后自动播放的下一个动画
    pub(crate) anim_next: HashMap<S, S>,

    /// 每帧的物理效果 key 为动画名称
    pub(crate) anim_physics: HashMap<S, PhyEff>,

    // ActionExitLogic 的参数使用【泛型方式】去实现的话需要如下实现
    // 让编译器以为使用了该泛型 零成本 （实例化时直接 `_marker: std::marker::PhantomData,` ）
    // 若使用【关联类型】去实现 则无需这样做
    // （没有选择【关联类型】这个方案，因为关联类型本身不支持泛型，编写框架时受限，且支持泛型后能够将多个参数合并：动作的退出逻辑、行为的进入逻辑等）
    pub(crate) _marker: std::marker::PhantomData<ExitParam>,
}

impl<S, Event, ExitParam, ExitLogic, PhyEff> Action<S, Event, ExitParam, ExitLogic, PhyEff>
where
    S: FixedString,
    Event: ActionEvent,
    ExitLogic: ActionExitLogic<ExitParam>,
{
    pub fn new_empty(action_name: S, anim_first: S) -> Self {
        Self {
            action_name,
            event_enter: Vec::new(),
            event_exit: HashMap::new(),
            logic_exit: Vec::new(),
            action_priority: 0,
            action_switch_relation: HashMap::new(),
            anim_first,
            anim_next: HashMap::new(),
            anim_physics: HashMap::new(),
            _marker: Default::default(),
        }
    }

    /// return None if not exist
    pub fn fetch_next_action_name_by_event(&self, event: &Event) -> Option<&S> {
        self.event_exit.get(event)
    }

    /// 本动作可以切换到另一个动作
    ///
    /// 先判断自定义覆盖 后判断优先级 优先级相同也允许覆盖（反复击飞）
    pub fn can_switch_other_action(&self, other: &Self) -> bool {
        if let Some(can_cover) = self.action_switch_relation.get(&other.action_name) {
            *can_cover
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

    pub fn get_phy_eff_by_anim(&self, anim: &S) -> Option<&PhyEff> {
        self.anim_physics.get(anim)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub enum TmpActionEvent {
        JumpInstruction,
        DodgeInstruction,
        AttackInstruction,
    }

    impl ActionEvent for TmpActionEvent {}

    /// 动作的退出逻辑
    #[derive(Clone, Copy, Debug)]
    pub enum TmpActionExitLogic<S: FixedString> {
        /// 动画结束播放
        AnimFinished(S),
        OtherExitLogic,
    }

    impl<S: FixedString> ActionExitLogic<bool> for TmpActionExitLogic<S> {
        fn should_exit(&self, p: &bool) -> bool {
            *p
        }
    }

    fn gen_for_test()
    -> Action<&'static str, TmpActionEvent, bool, TmpActionExitLogic<&'static str>, (f64, f64)>
    {
        Action {
            action_name: "attack",
            event_enter: Vec::from([TmpActionEvent::AttackInstruction]),
            event_exit: HashMap::from([
                (TmpActionEvent::AttackInstruction, "twice_atk"),
                (TmpActionEvent::JumpInstruction, "jump_atk"),
            ]),
            logic_exit: Vec::from([
                (TmpActionExitLogic::AnimFinished("attack_end"), "idle"),
                (TmpActionExitLogic::OtherExitLogic, "other"),
            ]),
            action_priority: 1,
            action_switch_relation: HashMap::from([
                ("be_knocked_down", true),
                ("burning", false), // 燃烧
            ]),
            anim_first: "attack_begin",
            anim_next: HashMap::from([
                ("attack_begin", "attack_middle"),
                ("attack_middle", "attack_end"),
            ]),
            anim_physics: HashMap::from([
                ("attack_begin", (1.0, 0.0)),
                ("attack_middle", (1.0, 0.0)),
                ("attack_end", (1.0, 0.0)),
            ]),
            _marker: Default::default(),
        }
    }

    #[test]
    fn test_event_getter() {
        let test_action = gen_for_test();
        assert_eq!(test_action.action_name, "attack");
        let res = test_action.fetch_next_action_name_by_event(&TmpActionEvent::AttackInstruction);
        assert_eq!(res, Some(&"twice_atk"));
        let res = test_action.fetch_next_action_name_by_event(&TmpActionEvent::DodgeInstruction);
        assert_eq!(res, None);
    }

    #[test]
    fn test_action_priority() {
        let test_action = gen_for_test();
        // by action_priority
        let empty_action = Action::new_empty("tmp", "tmp");
        assert!(!test_action.can_switch_other_action(&empty_action));
        // by can_cover
        let mut test2 = gen_for_test();
        test2.action_name = "be_knocked_down";
        assert!(test_action.can_switch_other_action(&test2));
        test2.action_name = "burning";
        assert!(!test_action.can_switch_other_action(&test2));
    }
}

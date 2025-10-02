use std::collections::HashMap;

use crate::cores::unify_type::FixedString;

/// 动作 纯数据 实现固定效果
#[derive(Clone, Debug)]
pub struct Action<S, Event, ExitLogic, PhyEff>
where
    S: FixedString,
    Event: Clone + std::fmt::Debug + Eq + std::hash::Hash + PartialEq,
    ExitLogic: ActionExitLogic + Clone + std::fmt::Debug,
{
    /// 动作名称
    pub action_name: S,

    /// 本动作的触发事件（指令与信号）
    pub trigger: Vec<Event>,

    /// 每帧执行退出逻辑判断是否进行下一个动作（指令输入也可在这里定义）
    pub tick_to_next_action: Vec<(ExitLogic, S)>,
    /// 事件触发的下一个动作（一般不包括指令）
    pub trigger_to_next_action: HashMap<Event, S>,

    /// 动作优先级
    pub action_priority: i64,
    /// 动作自定义覆盖关系 true能被其他覆盖 false不能
    pub action_switch_relation: HashMap<S, ActionCanCover>,

    /// 初始播放的动画名称
    pub anim_first: S,
    /// 动画结束后自动播放的下一个动画
    pub anim_next: HashMap<S, S>,

    /// 每帧的物理效果 key 为动画名称
    pub anim_physics: HashMap<S, PhyEff>,
    /// 事件响应时的物理效果
    pub trigger_physics: HashMap<Event, PhyEff>,
    /*
    // ActionExitLogic 的参数使用【泛型方式】去实现的话需要如下实现
    // 若使用【关联类型】去实现（当前选择这个方案） 则无需这样做
    // 让编译器以为使用了该泛型 零成本 （实例化时直接 `_marker: std::marker::PhantomData,` ）
    _marker: std::marker::PhantomData<ExitParam>,
     */
}

pub trait ActionExitLogic {
    type ExitParam;
    fn should_exit(p: &Self::ExitParam) -> bool;
}

#[derive(Clone, Copy, Debug)]
pub struct ActionCanCover(pub bool);

impl From<bool> for ActionCanCover {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl<S, Event, ExitLogic, PhyEff> Action<S, Event, ExitLogic, PhyEff>
where
    S: FixedString,
    Event: Clone + std::fmt::Debug + Eq + std::hash::Hash + PartialEq,
    ExitLogic: ActionExitLogic + Clone + std::fmt::Debug,
{
    pub fn new_empty(action_name: S, anim_first: S) -> Self {
        Self {
            action_name,
            trigger: Vec::new(),
            tick_to_next_action: Vec::new(),
            trigger_to_next_action: HashMap::new(),
            action_priority: 0,
            action_switch_relation: HashMap::new(),
            anim_first,
            anim_next: HashMap::new(),
            anim_physics: HashMap::new(),
            trigger_physics: HashMap::new(),
        }
    }

    /// 调用方去迭代 exit_logic
    pub fn exit_logic_slice(&self) -> &[(ExitLogic, S)] {
        &self.tick_to_next_action
    }

    /// return None if should not trigger other action
    pub fn should_trigger_to_next_action(&self, trigger: &Event) -> Option<&S> {
        self.trigger_to_next_action.get(trigger)
    }

    /// 本动作可以切换到另一个动作
    /// 
    /// 先判断自定义覆盖 后判断优先级 优先级相同也允许覆盖
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

    pub fn get_phy_eff_by_anim(&self, anim: &S) -> Option<&PhyEff> {
        self.anim_physics.get(anim)
    }

    pub fn get_phy_eff_by_trigger(&self, trigger: &Event) -> Option<&PhyEff> {
        self.trigger_physics.get(trigger)
    }
}

#[cfg(test)]
mod tests {
    use super::super::action_impl::{ActionBaseEvent, ActionBaseExitLogic};

    use super::*;

    impl ActionExitLogic for ActionBaseExitLogic {
        type ExitParam = bool;

        fn should_exit(p: &Self::ExitParam) -> bool {
            *p
        }
    }

    fn gen_for_test() -> Action<&'static str, ActionBaseEvent, ActionBaseExitLogic, (f64, f64)> {
        Action {
            action_name: "attack",
            trigger: Vec::from([ActionBaseEvent::AttackInstruction]),
            tick_to_next_action: Vec::from([
                (ActionBaseExitLogic::AnimFinished, "idle"),
                (ActionBaseExitLogic::WantMove(0.6), "move"),
            ]),
            trigger_to_next_action: HashMap::from([
                (ActionBaseEvent::AttackInstruction, "twice_atk"),
                (ActionBaseEvent::JumpInstruction, "jump_atk"),
            ]),
            action_priority: 1,
            action_switch_relation: HashMap::from([
                ("be_knocked_down", true.into()),
                ("burning", false.into()), // 燃烧
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
            trigger_physics: HashMap::from([(ActionBaseEvent::JumpInstruction, (0.0, 5.0))]),
        }
    }

    #[test]
    fn test_func() {
        let test_action = gen_for_test();

        let res = test_action.should_trigger_to_next_action(&ActionBaseEvent::AttackInstruction);
        assert_eq!(res.unwrap(), &"twice_atk");
        let res = test_action.should_trigger_to_next_action(&ActionBaseEvent::DodgeInstruction);
        assert_eq!(res, None);

        // by action_priority
        let empty_action = Action::new_empty("tmp", "tmp");
        assert!(!test_action.should_switch_other_action(&empty_action));
        // by can_cover
        let mut test2 = gen_for_test();
        test2.action_name = "be_knocked_down";
        assert!(test_action.should_switch_other_action(&test2));
        test2.action_name = "burning";
        assert!(!test_action.should_switch_other_action(&test2));
    }
}

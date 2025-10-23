use crate::cores::unify_type::FixedString;

/// 若有必要可将角色动画分层（如上半身下半身组合动画），动作系统的逻辑保持单一仍然只返回一个动画
#[derive(Debug, Default)]
pub struct FrameEff<S: FixedString> {
    /// 动画名称（主动画） 始终不应为空
    pub anim_name: S, // pub-external
    /// 特效（不区分视觉听觉），可以为空
    pub special_eff: S, // pub-external
    /// 当前动画能否自由转向
    pub not_turn_back: bool, // pub-external
}

// 由于 S 是泛型，所以无法实现 TryFrom （具体原因存疑，反正就是有冲突，怀疑可能是编译器太过于严格）
impl<S: FixedString> From<S> for FrameEff<S> {
    fn from(value: S) -> Self {
        Self {
            anim_name: value,
            special_eff: Default::default(),
            not_turn_back: false,
        }
    }
}

impl<S: FixedString> FrameEff<S> {
    pub fn is_legal(&self) -> bool {
        self.anim_name.is_legal()
    }

    pub fn try_from_action_anim(s: S) -> Option<Self> {
        let frame_eff = FrameEff {
            anim_name: s,
            special_eff: Default::default(),
            not_turn_back: true, // 动作系统中的动画始终无法自由转向
        };
        if frame_eff.is_legal() {
            Some(frame_eff)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_is_legal() {
        let a = FrameEff::try_from_action_anim("");
        assert!(a.is_none());

        let b = FrameEff::try_from_action_anim(" ");
        assert!(b.is_some());
    }
}

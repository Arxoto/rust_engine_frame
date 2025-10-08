use crate::cores::unify_type::FixedString;

/// 若有必要可将角色动画分层（如上半身下半身组合动画），动作系统的逻辑保持单一仍然只返回一个动画
#[derive(Debug, Default)]
pub struct FrameEff<S: FixedString> {
    pub(crate) anim_name: S,
}

// 由于 S 是泛型，所以无法实现 TryFrom （具体原因存疑，反正就是有冲突，怀疑可能是编译器太过于严格）
impl<S: FixedString> From<S> for FrameEff<S> {
    fn from(value: S) -> Self {
        Self { anim_name: value }
    }
}

impl<S: FixedString> FrameEff<S> {
    pub fn is_legal(&self) -> bool {
        self.anim_name.is_legal()
    }

    pub fn try_new(s: S) -> Option<Self> {
        let frame_eff = FrameEff::from(s);
        if frame_eff.is_legal() {
            Some(frame_eff)
        } else {
            None
        }
    }
}

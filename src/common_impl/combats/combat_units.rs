//! 战斗属性
//!
//! 字段设为 pub 支持拆分平铺到实体中去
//!
//! 参考经典三维：
//! - health 血量
//! - stamina 耐力，这里置换为平衡
//! - magicka/mana 法力，这里置换为能量（气势）
//!
//! 特殊属性条
//! - 累积进度条 被元素系统控制
//! - 系统内部定义属性条满时正常（为与 [`crate::base_lib::eff_attr_prop::effects::EffectMean`] 一致）
//! - 界面视觉上可翻转成空时正常
//! - 类别
//!   - Entropy 熵（炎热寒冷）
//!   - Electric 电势能
//!
//! todo 后续系统性实现
//! - 实现角色周期性效果
//! - 生命值以最大值的百分比进行恢复
//! - 平衡以固定值进行恢复 受击后延迟一段时间继续恢复
//! - 能量以固定值削减 增长后延迟一段时间继续削减

use strum_macros::EnumIter;

use crate::base_lib::{
    cores::{timers::static_timer::StaticTimer, unify_types::FixedName},
    eff_attr_prop::{
        multi_prop::MultiPropLayer, prop_bounds_eff::PropBoundsEffect, props::Prop,
        upsert_container::UpsertContainer,
    },
};

/// 生存属性类型（生命值、护盾）
///
/// 生命值护盾的层级关系的值是业务约定
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum SurvivalPropLayer {
    Health,
    ShieldSubstitute,
    ShieldDefence,
    ShieldArcane,
}

impl MultiPropLayer for SurvivalPropLayer {
    fn get_next(&self) -> Self {
        match self {
            SurvivalPropLayer::Health => Self::Health,
            SurvivalPropLayer::ShieldSubstitute => Self::Health,
            SurvivalPropLayer::ShieldDefence => Self::ShieldSubstitute,
            SurvivalPropLayer::ShieldArcane => Self::ShieldSubstitute,
        }
    }

    fn get_layer(&self) -> u8 {
        match self {
            SurvivalPropLayer::Health => 0,
            SurvivalPropLayer::ShieldSubstitute => 1,
            SurvivalPropLayer::ShieldDefence => 2,
            SurvivalPropLayer::ShieldArcane => 2,
        }
    }
}

/// 替身护盾 被伤害系统控制；
pub struct ShieldSubstitute(pub Prop);

/// 防护护盾 被伤害系统控制；
pub struct ShieldDefence(pub Prop);

/// 奥术护盾 被伤害系统控制；
pub struct ShieldArcane(pub Prop);

/// 血量 被伤害系统控制； 基础值被【气力】的基础值影响
pub struct Health(pub Prop);

/// 耐力（平衡） 被冲击韧性系统控制； 基础值和最大值固定，清空时触发倒地
pub struct Stamina(pub Prop);

/// 能量（气势） 被战时评价系统控制； 基础值被【信念】的基础值和能级系统影响
pub struct Magicka(pub Prop);

/// 类型别名 资源上下限效果的容器
type PropBoundsEffs<S> = UpsertContainer<PropBoundsEffect<S, StaticTimer>>;

pub struct ShieldSubstituteEffs<S: FixedName>(pub PropBoundsEffs<S>);
pub struct ShieldDefenceEffs<S: FixedName>(pub PropBoundsEffs<S>);
pub struct ShieldArcaneEffs<S: FixedName>(pub PropBoundsEffs<S>);
pub struct HealthEffs<S: FixedName>(pub PropBoundsEffs<S>);
pub struct StaminaEffs<S: FixedName>(pub PropBoundsEffs<S>);
pub struct MagickaEffs<S: FixedName>(pub PropBoundsEffs<S>);

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use crate::base_lib::eff_attr_prop::multi_prop::multi_prop_system;

    use super::*;

    /// 在单元测试中检查以避免运行时开销
    #[test]
    fn check_survival_layer() {
        let all_svv_types: Vec<_> = SurvivalPropLayer::iter().collect();
        for ele in all_svv_types {
            multi_prop_system::check_multi_prop_layer(ele);
        }
    }
}

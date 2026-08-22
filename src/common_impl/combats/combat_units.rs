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
//! - 系统内部定义属性条满时正常（为与 [`crate::base_lib::eff_attr::effects::EffectMean`] 一致）
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
    eff_attr::{
        attr_layers::AttrLayerType, bound_attr_effs::BoundAttrEff, bound_attrs::BoundAttr,
        bounded_attrs::BoundedAttr, upsert_container::UpsertContainer,
    },
};

/// 生存属性类型（生命值、护盾）
///
/// 生命值护盾的层级关系的值是业务约定
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum SurvivalAttrLayer {
    Health,
    ShieldSubstitute,
    ShieldDefence,
    ShieldArcane,
}

impl AttrLayerType for SurvivalAttrLayer {
    fn get_next(&self) -> Self {
        match self {
            SurvivalAttrLayer::Health => Self::Health,
            SurvivalAttrLayer::ShieldSubstitute => Self::Health,
            SurvivalAttrLayer::ShieldDefence => Self::ShieldSubstitute,
            SurvivalAttrLayer::ShieldArcane => Self::ShieldSubstitute,
        }
    }

    fn get_layer(&self) -> u8 {
        match self {
            SurvivalAttrLayer::Health => 0,
            SurvivalAttrLayer::ShieldSubstitute => 1,
            SurvivalAttrLayer::ShieldDefence => 2,
            SurvivalAttrLayer::ShieldArcane => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
pub enum EnergyAttrLayer {
    Magicka,
    ExternalEnergy,
}

impl AttrLayerType for EnergyAttrLayer {
    fn get_next(&self) -> Self {
        match self {
            EnergyAttrLayer::Magicka => Self::Magicka,
            EnergyAttrLayer::ExternalEnergy => Self::Magicka,
        }
    }

    fn get_layer(&self) -> u8 {
        match self {
            EnergyAttrLayer::Magicka => 0,
            EnergyAttrLayer::ExternalEnergy => 1,
        }
    }
}

impl EnergyAttrLayer {
    /// 能量消耗统一路径
    #[inline]
    pub fn start_at() -> Self {
        Self::ExternalEnergy
    }
}

/// 替身护盾 被伤害系统控制；
pub struct ShieldSubstitute(pub BoundedAttr);

/// 防护护盾 被伤害系统控制；
pub struct ShieldDefence(pub BoundedAttr);

/// 奥术护盾 被伤害系统控制；
pub struct ShieldArcane(pub BoundedAttr);

/// 血量 被伤害系统控制； 基础值被【气力】的基础值影响
pub struct Health(pub BoundedAttr);

/// 耐力（平衡） 被冲击韧性系统控制； 基础值和最大值固定，清空时触发倒地
pub struct Stamina(pub BoundedAttr);

/// 魔能（气势） 被战时评价系统控制； 基础值被【信念】的基础值和能级系统影响
pub struct Magicka(pub BoundedAttr);

/// 外部能源 环境逸散的自由态能量
pub struct ExternalEnergy(pub BoundedAttr);

// region: 批量定义

pub struct ShieldSubstituteUpper(pub BoundAttr);
pub struct ShieldDefenceUpper(pub BoundAttr);
pub struct ShieldArcaneUpper(pub BoundAttr);
pub struct HealthUpper(pub BoundAttr);
pub struct StaminaUpper(pub BoundAttr);
pub struct MagickaUpper(pub BoundAttr);
pub struct HealthLower(pub BoundAttr);

/// 类型别名 资源上下限效果的容器
type BoundAttrEffs<S> = UpsertContainer<BoundAttrEff<S, StaticTimer>>;

pub struct ShieldSubstituteUpperEffs<S: FixedName>(pub BoundAttrEffs<S>);
pub struct ShieldDefenceUpperEffs<S: FixedName>(pub BoundAttrEffs<S>);
pub struct ShieldArcaneUpperEffs<S: FixedName>(pub BoundAttrEffs<S>);
pub struct HealthUpperEffs<S: FixedName>(pub BoundAttrEffs<S>);
pub struct StaminaUpperEffs<S: FixedName>(pub BoundAttrEffs<S>);
pub struct MagickaUpperEffs<S: FixedName>(pub BoundAttrEffs<S>);
pub struct HealthLowerEffs<S: FixedName>(pub BoundAttrEffs<S>);

// endregion

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use crate::base_lib::eff_attr::attr_layers::attr_layer_system;

    use super::*;

    /// 在单元测试中检查以避免运行时开销
    #[test]
    fn check_survival_layer() {
        let all_svv_types: Vec<_> = SurvivalAttrLayer::iter().collect();
        for ele in all_svv_types {
            attr_layer_system::check_attr_layer(ele);
        }
    }

    #[test]
    fn check_energy_layer() {
        let layers: Vec<_> = EnergyAttrLayer::iter().collect();
        for ele in layers {
            attr_layer_system::check_attr_layer(ele);
        }
    }
}

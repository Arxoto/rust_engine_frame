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

use crate::base_lib::eff_attr::{
    bound_attr_modifiers::BoundAttrModifier, bound_attrs::BoundAttr, bounded_attrs::BoundedAttr,
    modifier_collections::ModifierCollection,
};

pub use super::energies::Magicka;
pub use super::energies::MagickaUpper;
pub use super::energies::MagickaUpperEffs;

pub use super::energies::ExternalEnergy;

pub use super::damages::Health;
pub use super::damages::HealthLower;
pub use super::damages::HealthLowerEffs;
pub use super::damages::HealthUpper;
pub use super::damages::HealthUpperEffs;

pub use super::damages::ShieldArcane;
pub use super::damages::ShieldArcaneUpper;
pub use super::damages::ShieldArcaneUpperEffs;

pub use super::damages::ShieldDefence;
pub use super::damages::ShieldDefenceUpper;
pub use super::damages::ShieldDefenceUpperEffs;

pub use super::damages::ShieldSubstitute;
pub use super::damages::ShieldSubstituteUpper;
pub use super::damages::ShieldSubstituteUpperEffs;

/// 耐力（平衡） 被冲击韧性系统控制； 基础值和最大值固定，清空时触发倒地
pub struct Stamina(pub BoundedAttr);
pub struct StaminaUpper(pub BoundAttr);
pub struct StaminaUpperEffs(pub ModifierCollection<BoundAttrModifier>);

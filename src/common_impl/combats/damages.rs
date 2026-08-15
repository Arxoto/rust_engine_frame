use crate::base_lib::{
    cores::{timers::static_timer::StaticTimer, unify_types::FixedName},
    eff_attr_prop::{
        prop_bounds_eff::PropBoundsEffect, props::Prop, upsert_container::UpsertContainer,
    },
};

// region: 伤害相关属性定义

/// 类型别名 持久效果的容器
type PropBoundsEffs<S> = UpsertContainer<PropBoundsEffect<S, StaticTimer>>;

/// 血量（战时动态值） 变化由伤害系统控制，基础值受气力的基础值影响
pub struct Health(pub Prop);

/// 血量（战时动态值） 上下限效果
pub struct HealthEffs<S: FixedName>(pub PropBoundsEffs<S>);

/// 替身护盾 受伤害系统控制
pub struct ShieldSubstitute(pub Prop);

/// 替身护盾 上下限效果
pub struct ShieldSubstituteEffs<S: FixedName>(pub PropBoundsEffs<S>);

/// 防护护盾 受伤害系统控制
pub struct ShieldDefence(pub Prop);

/// 防护护盾 上下限效果
pub struct ShieldDefenceEffs<S: FixedName>(pub PropBoundsEffs<S>);

/// 奥术护盾 受伤害系统控制
pub struct ShieldArcane(pub Prop);

/// 奥术护盾 上下限效果
pub struct ShieldArcaneEffs<S: FixedName>(pub PropBoundsEffs<S>);

// endregion

#[derive(Debug, Clone, Copy)]
pub enum DamageType {
    /// 因果论的真实伤害
    KarmaTruth,
    /// 物理剪切 尖锐
    PhysicsShear,
    /// 物理冲击 沉重
    PhysicsImpact,
    /// 魔法奥术
    MagickaArcane,

    /// 防护破盾
    BrokeShieldDefence,
    /// 奥术破盾
    BrokeShieldArcane,
}

impl DamageType {
    /// 能否对血量造成伤害（剔除破盾类型）
    pub fn hurt_to_health(&self) -> bool {
        match self {
            DamageType::KarmaTruth => true,
            DamageType::PhysicsShear => true,
            DamageType::PhysicsImpact => true,
            DamageType::MagickaArcane => true,
            DamageType::BrokeShieldDefence => false,
            DamageType::BrokeShieldArcane => false,
        }
    }
}

/// 伤害信息，表示每次伤害造成的影响
#[derive(Debug)]
pub struct DamageInfo {
    /// 伤害类型为“伤害”时表示是否致死，伤害类型为破盾时表示是否成功击穿防御
    pub broken: bool,
    /// 造成的伤害（致死时伤害可能大于实际扣血值）
    pub damage: f64,
}


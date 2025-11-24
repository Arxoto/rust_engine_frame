use crate::{
    attrs::dyn_prop::DynProp,
    combat::{combat_units::CombatUnit, damages::HealthShieldComponent},
    cores::unify_type::FixedName,
};

/// 战斗属性
/// 
/// 参考经典三维：
/// - health 血量
/// - stamina 耐力，这里置换为平衡
/// - magicka/mana 法力，这里置换为能量（气势）
pub struct CombatEntity<S: FixedName = String> {
    /// 能量（气势）
    pub(crate) magicka: DynProp<S>,

    /// 平衡 战时动态值 变化由冲击韧性系统控制
    ///
    /// 基础值和最大值固定 清空时触发倒地
    pub(crate) stamina: DynProp<S>,

    /// 血量和护盾
    pub(crate) health_and_shield: HealthShieldComponent<S>,

    /// 熵（炎热寒冷） 累积进度条 受元素系统控制
    ///
    /// 为统一区分增益效果的，编码时统一规定所有属性条满时正常，归零异常，满时异常的累积进度条仅体现在视觉表达上
    pub(crate) bar_entropy: DynProp<S>,
    /// 电势能 累积进度条 受元素系统控制
    ///
    /// 为统一区分增益效果的，编码时统一规定所有属性条满时正常，归零异常，满时异常的累积进度条仅体现在视觉表达上
    pub(crate) bar_eelectric: DynProp<S>,
}

impl<S: FixedName> CombatEntity<S> {
    pub fn new(
        magicka_base: f64,
        magicka_scale: f64,
        health_base: f64,
        health_scale: f64,
        combat_unit: &CombatUnit<S>,
    ) -> CombatEntity<S> {
        let magicka_max = magicka_base + magicka_scale * combat_unit.belief.get_origin();
        let health_value = health_base + health_scale * combat_unit.belief.get_origin();
        CombatEntity {
            magicka: DynProp::new(0.0, magicka_max, 0.0),
            stamina: DynProp::new_by_max(100.0),

            health_and_shield: HealthShieldComponent {
                health: DynProp::new_by_max(health_value),
                shield_substitute: DynProp::new_by_max(0.0),
                shield_defence: DynProp::new_by_max(0.0),
                shield_arcane: DynProp::new_by_max(0.0),
            },

            bar_entropy: DynProp::new_by_max(100.0),
            bar_eelectric: DynProp::new_by_max(100.0),
        }
    }
}

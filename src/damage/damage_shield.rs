use crate::{attrs::dyn_prop::DynProp, cores::unify_type::FixedName};

/// 护盾类型
pub trait AsShieldType {}

/// 带护盾的生命值
pub trait AsShieldHealth<T: AsShieldType, S: FixedName = String> {
    fn get_shield(&mut self, shield_type: T) -> &mut DynProp<S>;
}

/// 伤害类型
pub trait AsDamageType<T: AsShieldType> {
    /// 映射为一系列的护盾类型 用于抵挡伤害
    ///
    /// 仅表示伤害的逻辑顺序 不表达具体如何抵挡（抵挡直至清空/每次抵挡具有上限）
    fn map_defence_types(&self) -> &[T];
}

#[allow(dead_code)]
#[cfg(test)]
mod tests {
    use super::*;

    enum ShieldType {
        /// 生命
        Health,
        /// 防御
        Defence,

        /// 因果
        Karma,

        /// 魔法
        Magicka,
    }

    enum DamageType {
        /// 真实伤害
        Truth,
        /// 物理切割（尖锐）
        PhysicsSlice,
        /// 物理冲击（沉重）
        PhysicsImpact,

        /// 因果
        Karma,

        /// 魔法
        Magicka,
        // /// 奥术
        // Arcane,
        // /// 元素
        // Elemental,
        // /// 熵 炎热寒冷
        // Entropy,
        // /// 电
        // Electric,
    }

    impl AsShieldType for ShieldType {}

    impl AsDamageType<ShieldType> for DamageType {
        fn map_defence_types(&self) -> &[ShieldType] {
            match self {
                DamageType::Truth => &[ShieldType::Health],
                DamageType::PhysicsSlice => &[ShieldType::Defence, ShieldType::Health],
                DamageType::PhysicsImpact => &[ShieldType::Health],
                DamageType::Karma => &[ShieldType::Karma],
                DamageType::Magicka => &[ShieldType::Magicka, ShieldType::Health],
            }
        }
    }

    #[test]
    fn map_defence_type() {
        let defence_types = DamageType::PhysicsSlice.map_defence_types();
        assert_eq!(defence_types.iter().count(), 2);
    }
}

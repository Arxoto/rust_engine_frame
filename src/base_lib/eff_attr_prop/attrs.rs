use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr_prop::attr_eff::{AttrEffect, AttrModifier},
};

/// attribute 属性 一般用作角色属性值 可被效果影响
///
/// 另一种风格是将所有基础属性展开平铺，每个效果直接修改对应属性，灵活但复杂
#[derive(Debug, Default)]
pub struct Attr {
    origin: f64,
    current: f64,
}

impl Attr {
    pub fn new(origin: f64) -> Self {
        Self {
            origin,
            current: origin,
        }
    }

    pub fn get_origin(&self) -> f64 {
        self.origin
    }

    pub fn get_current(&self) -> f64 {
        self.current
    }

    /// 刷新属性，在效果更新后
    pub fn refresh_value<'a, S: FixedName + 'a, Timer: 'a>(
        &mut self,
        effs: impl Iterator<Item = &'a AttrEffect<S, Timer>>,
    ) {
        let mut attr_modifier = AttrModifier::default();

        for ele in effs {
            attr_modifier.reduce(ele);
        }

        self.apply_modify(&attr_modifier);
    }

    pub(super) fn apply_modify(&mut self, am: &AttrModifier) {
        self.current = am.apply_modify(self.origin)
    }
}

#[cfg(test)]
mod tests {
    use crate::base_lib::{
        cores::timers::tick_timer::TickTimer,
        eff_attr_prop::{attr_eff::AttrEffectType, effects::Effect},
    };

    use super::*;

    fn make_eff(eff_type: AttrEffectType, eff_value: f64) -> AttrEffect<String, TickTimer> {
        AttrEffect::new(
            eff_type,
            Effect::new_form("from", "eff", eff_value),
            TickTimer::inf(),
        )
    }

    /// 初始值与当前值一致
    #[test]
    fn test_new() {
        let attr = Attr::new(100.0);
        assert_eq!(attr.get_origin(), 100.0);
        assert_eq!(attr.get_current(), 100.0);
    }

    /// Default 为 0
    #[test]
    fn test_default() {
        let attr = Attr::default();
        assert_eq!(attr.get_origin(), 0.0);
        assert_eq!(attr.get_current(), 0.0);
    }

    /// 空效果刷新后当前值回落到基础值
    #[test]
    fn test_refresh_empty_restores_origin() {
        let mut attr = Attr::new(100.0);
        let effs: Vec<AttrEffect<String, TickTimer>> = Vec::new();
        attr.refresh_value(effs.iter());
        assert_eq!(attr.get_current(), 100.0);
    }

    /// 单个基础加法效果
    #[test]
    fn test_refresh_basic_add() {
        let mut attr = Attr::new(100.0);
        let effs = vec![make_eff(AttrEffectType::BasicAdd, 10.0)];
        attr.refresh_value(effs.iter());
        assert_eq!(attr.get_current(), 110.0);
    }

    /// 同类型效果累加
    #[test]
    fn test_refresh_multi_same_type() {
        let mut attr = Attr::new(100.0);
        let effs = vec![
            make_eff(AttrEffectType::BasicAdd, 1.0),
            make_eff(AttrEffectType::BasicAdd, 2.0),
        ];
        attr.refresh_value(effs.iter());
        assert_eq!(attr.get_current(), 103.0);
    }

    /// 混合效果的完整公式
    #[test]
    fn test_refresh_mixed_formula() {
        let mut attr = Attr::new(100.0);
        let effs = vec![
            make_eff(AttrEffectType::BasicAdd, 10.0),
            make_eff(AttrEffectType::BasicPer, 0.5),
            make_eff(AttrEffectType::FinalPer, 0.25),
            make_eff(AttrEffectType::FinalMul, 2.0),
        ];
        attr.refresh_value(effs.iter());
        // (10 + 100*1.5) * 1.25 * 2 = 400
        assert_eq!(attr.get_current(), 400.0);
    }

    /// 重复刷新不会叠加（每次从基础值重算）
    #[test]
    fn test_refresh_is_recompute_not_accumulate() {
        let mut attr = Attr::new(100.0);
        let effs = vec![make_eff(AttrEffectType::BasicAdd, 10.0)];
        attr.refresh_value(effs.iter());
        attr.refresh_value(effs.iter());
        assert_eq!(attr.get_current(), 110.0);
    }

    /// 边界值：最终乘法为 0 时当前值恒为 0
    #[test]
    fn test_refresh_final_mul_zero() {
        let mut attr = Attr::new(100.0);
        let effs = vec![make_eff(AttrEffectType::FinalMul, 0.0)];
        attr.refresh_value(effs.iter());
        assert_eq!(attr.get_current(), 0.0);
    }

    /// 边界值：负效果使当前值低于基础值
    #[test]
    fn test_refresh_negative_effects() {
        let mut attr = Attr::new(100.0);
        let effs = vec![
            make_eff(AttrEffectType::BasicAdd, -30.0),
            make_eff(AttrEffectType::BasicPer, -0.5),
        ];
        attr.refresh_value(effs.iter());
        // (-30 + 100*0.5) * 1 * 1 = 20
        assert_eq!(attr.get_current(), 20.0);
    }
}

//! 复合属性实现 [`apply_alter`] [`apply_alter_safety`]
//!
//! 完整逻辑
//! - 首先对修改效果进行排序，排序标准参考 [`super::attr_layers`]
//! - 然后根据修改效果类型依次对目标属性进行修改
//!
//! P.S. 这里为简化逻辑，效果遍历的逻辑放在调用方实现

use crate::base_lib::eff_attr::{
    attr_layers::{AttrLayerEffTarget, AttrLayerEffTargetIter},
    bound_attrs::BoundRange,
    bounded_attrs::BoundedAttr,
};

pub trait CompoundAttr<EffTarget: AttrLayerEffTarget> {
    fn get_attr_mut(
        &mut self,
        target_layer: <EffTarget as AttrLayerEffTarget>::Layer,
    ) -> &mut BoundedAttr;
}

pub trait CompoundAttrBound<EffTarget: AttrLayerEffTarget> {
    fn gen_bound_range(&self, target_layer: <EffTarget as AttrLayerEffTarget>::Layer)
    -> BoundRange;
}

/// 直接修改
pub fn apply_alter<EffTarget>(
    compound_attr: &mut impl CompoundAttr<EffTarget>,
    compound_attr_bound: &impl CompoundAttrBound<EffTarget>,
    eff_targets: EffTarget,
    mut delta_val: f64,
) where
    EffTarget: AttrLayerEffTarget,
{
    let layer_iter = AttrLayerEffTargetIter::from(eff_targets);
    for current_layer in layer_iter {
        let attr = compound_attr.get_attr_mut(current_layer);
        let bound_range = compound_attr_bound.gen_bound_range(current_layer);

        let old_val = attr.get_pending_value();
        attr.apply_alter(delta_val);
        attr.clamp_by(bound_range);
        let new_val = attr.get_pending_value();

        if new_val > 0.0 {
            // 如果最小值强制护盾没有击破，那么不应该继续往下
            // 对于增益效果，就算类型允许向下传递，也会被强制截断
            // 就算设计上想要允许传递，效果表现为优先填充护盾，护盾满了再治疗生命值，这个设计本身也很奇怪
            return;
        }

        let diff_val = new_val - old_val;
        delta_val -= diff_val;
    }
}

/// 安全修改
pub fn apply_alter_safety<EffTarget>(
    compound_attr: &mut impl CompoundAttr<EffTarget>,
    compound_attr_bound: &impl CompoundAttrBound<EffTarget>,
    eff_targets: EffTarget,
    mut delta_val: f64,
    must_ge: f64,
) -> bool
where
    EffTarget: AttrLayerEffTarget,
{
    let bottom_layer = eff_targets.stop_at();
    let layer_iter = AttrLayerEffTargetIter::from(eff_targets);
    for current_layer in layer_iter {
        let attr = compound_attr.get_attr_mut(current_layer);
        let bound_range = compound_attr_bound.gen_bound_range(current_layer);

        let old_val = attr.get_pending_value();
        let new_val = attr.calc_clamped_pending(bound_range, delta_val);

        // 最后一轮迭代，直接判断
        if current_layer == bottom_layer {
            if new_val >= must_ge {
                // 允许执行
                break;
            } else {
                return false;
            }
        }

        if new_val > 0.0 {
            // 同修改逻辑，中断循环
            // 没有击破护盾，不会因为此效果导致属性值小于预期值（因为并没有对这个属性生效）
            // 因此无论如何都执行这个修改
            break;
        }

        let diff_val = new_val - old_val;
        delta_val -= diff_val;
    }

    // 逻辑复杂，直接调用应用修改的函数
    apply_alter(compound_attr, compound_attr_bound, eff_targets, delta_val);
    true
}

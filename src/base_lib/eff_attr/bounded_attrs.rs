use crate::base_lib::eff_attr::{bound_attrs::BoundAttr, modifier_collections::ModifiableAttr};

#[derive(Debug)]
pub struct AttrAlterResult {
    /// 实际生效值
    pub diff_val: f64,
}

/// 有界属性的约束值，用于自动转换
#[derive(Debug)]
pub struct BoundValue(f64);

impl BoundValue {
    #[inline]
    pub fn get_value(v: impl Into<BoundValue>) -> f64 {
        let value: Self = v.into();
        value.0
    }

    #[inline]
    pub fn clamp(lower: impl Into<BoundValue>, upper: impl Into<BoundValue>, v: f64) -> f64 {
        let lower = BoundValue::get_value(lower);
        let upper = BoundValue::get_value(upper);
        lower.max(upper.min(v))
    }
}

impl From<f64> for BoundValue {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl From<&BoundAttr> for BoundValue {
    fn from(value: &BoundAttr) -> Self {
        Self(value.get_current())
    }
}

/// 有界属性，一般作为各种系统的结果，比如 “血量/蓝量”
#[derive(Debug)]
pub struct BoundedAttr {
    /// 快照值，每次修改前的快照，在一帧中保持不变
    snapshot: f64,
    /// 计算过程中的中间态
    pending: f64,
}

impl BoundedAttr {
    pub fn new(v: f64) -> Self {
        Self {
            snapshot: v,
            pending: v,
        }
    }

    pub fn get_snapshot_value(&self) -> f64 {
        self.snapshot
    }

    pub fn get_pending_value(&self) -> f64 {
        self.pending
    }

    pub fn commit_pending_value(&mut self) {
        self.snapshot = self.pending;
    }

    /// 钳制 应用上下界限
    ///
    /// - 一般情况下，应该是一帧内批量生效效果，然后单次钳制；
    /// - 特别的，对于复合属性，由于需要计算效果传递，因此必须每次计算都进行钳制；
    pub fn clamp_by(&mut self, lower: impl Into<BoundValue>, upper: impl Into<BoundValue>) {
        self.pending = BoundValue::clamp(lower, upper, self.pending);
    }

    /// 考虑到公式计算的复杂性，这里只支持输入具体值，不做统一的计算公式抽象
    pub fn apply_eff(&mut self, val: f64) {
        self.pending += val;
    }

    /// 只有当前值足够才会去应用效果（如法力不够则施放失败）
    pub fn apply_eff_checked(
        &mut self,
        lower: impl Into<BoundValue>,
        upper: impl Into<BoundValue>,
        val: f64,
        want_gt: f64,
    ) -> bool {
        let new_pending = self.pending + val;
        let new_clamped = BoundValue::clamp(lower, upper, new_pending);

        if new_clamped >= want_gt {
            self.pending = new_clamped;
            true
        } else {
            false
        }
    }
}

// todo 抽象 应用效果值 特征

// todo test clamp_by & apply_eff_checked

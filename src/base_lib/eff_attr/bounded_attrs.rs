use crate::base_lib::eff_attr::bound_attrs::BoundAttr;

#[derive(Debug)]
pub struct AttrAlterResult {
    /// 实际生效值
    pub diff_val: f64,
}

/// 有界属性的约束值，用于自动转换
#[derive(Debug)]
pub struct BoundValue(f64);

impl From<f64> for BoundValue {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl From<BoundAttr> for BoundValue {
    fn from(value: BoundAttr) -> Self {
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
    pub fn clamp_by<V1, V2>(&mut self, lower: V1, upper: V2)
    where
        BoundValue: From<V1>,
        BoundValue: From<V2>,
    {
        let lower = BoundValue::from(lower).0;
        let upper = BoundValue::from(upper).0;

        let old_v = self.pending;
        let new_v = lower.max(upper.min(old_v));
        self.pending = new_v;
    }

    /// 考虑到公式计算的复杂性，这里只支持输入具体值，不做统一的计算公式抽象
    pub fn apply_eff(&mut self, val: f64) {
        self.pending += val;
    }

    /// 只有当前值足够才会去应用效果（如法力不够则施放失败）
    pub fn apply_eff_checked<V>(&mut self, lower: V, val: f64, want_gt: f64) -> bool
    where
        BoundValue: From<V>,
    {
        let lower = BoundValue::from(lower).0;

        if lower.max(self.pending + val) >= want_gt {
            self.pending += val;
            true
        } else {
            false
        }
    }
}

// todo test clamp_by & apply_eff_checked

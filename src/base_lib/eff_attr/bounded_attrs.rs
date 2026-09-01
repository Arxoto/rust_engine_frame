use crate::base_lib::eff_attr::bound_attrs::{BoundRange, BoundValue};

pub trait Alterable {
    /// 考虑到公式计算的复杂性，这里只支持输入具体值，不做统一的计算公式抽象
    fn apply_alter(&mut self, delta_val: f64);
}

pub trait BoundedAlterable<BoundBy> {
    /// 钳制 应用上下界限
    ///
    /// - 一般情况下，应该是一帧内批量生效效果，然后单次钳制；
    /// - 特别的，对于复合属性，由于需要计算效果传递，因此必须每次计算都进行钳制；
    fn clamp_by(&mut self, bound_by: BoundBy);

    /// 只有当前值足够才会去应用效果（如法力不够则施放失败）
    fn apply_alter_checked(&mut self, bound_by: BoundBy, assert_ge: f64, delta_val: f64) -> bool;
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
}

impl Alterable for BoundedAttr {
    fn apply_alter(&mut self, delta_val: f64) {
        self.pending += delta_val;
    }
}

impl BoundedAlterable<BoundRange> for BoundedAttr {
    fn clamp_by(&mut self, bound_by: BoundRange) {
        self.pending = bound_by.clamp(self.pending);
    }

    fn apply_alter_checked(
        &mut self,
        bound_by: BoundRange,
        assert_ge: f64,
        delta_val: f64,
    ) -> bool {
        let new_pending = self.pending + delta_val;
        let new_clamped = bound_by.clamp(new_pending);

        if new_clamped >= assert_ge {
            self.pending = new_clamped;
            true
        } else {
            false
        }
    }
}

// todo test clamp_by & apply_eff_checked

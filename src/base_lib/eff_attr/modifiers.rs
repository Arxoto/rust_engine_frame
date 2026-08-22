pub(super) const ADDITION_BASE_LINE: f64 = 0.0;
pub(super) const PERCENT_BASE_LINE: f64 = 0.0;
pub(super) const MULT_BASE_LINE: f64 = 1.0;

/// 基准锚点值的修改器
///
/// 只允许在基础值上进行修改，具备极佳的“修改可预测性”
#[derive(Debug)]
pub(super) struct AnchorModifier {
    addition: f64,
    percent: f64,
}

impl Default for AnchorModifier {
    fn default() -> Self {
        Self {
            addition: ADDITION_BASE_LINE,
            percent: PERCENT_BASE_LINE,
        }
    }
}

impl AnchorModifier {
    pub fn reduce_add(&mut self, v: f64) {
        self.addition += v
    }

    pub fn reduce_pct(&mut self, v: f64) {
        self.percent += v
    }

    /// 计算公式 `base_value * (1 + b_per) + b_add`
    pub fn apply_modify(&self, v: f64) -> f64 {
        self.addition + (1.0 + self.percent) * v
    }
}

/// 聚合值的修改器
///
/// 支持多源公式合并，保证了“时间一致性”
#[derive(Debug)]
pub(super) struct AggregateModifier {
    basic: AnchorModifier,
    final_pct: f64,
    final_mult: f64,
}

impl Default for AggregateModifier {
    fn default() -> Self {
        Self {
            basic: Default::default(),
            final_pct: PERCENT_BASE_LINE,
            final_mult: MULT_BASE_LINE,
        }
    }
}

impl AggregateModifier {
    pub fn reduce_basic_add(&mut self, v: f64) {
        self.basic.reduce_add(v);
    }

    pub fn reduce_basic_pct(&mut self, v: f64) {
        self.basic.reduce_pct(v);
    }

    pub fn reduce_final_pct(&mut self, v: f64) {
        self.final_pct += v;
    }

    pub fn reduce_final_mult(&mut self, v: f64) {
        self.final_mult *= v;
    }

    /// 计算公式 `(base_value * (1 + b_per) + b_add) * (1 + f_per) * f_multi`
    pub fn apply_modify(&self, v: f64) -> f64 {
        self.basic.apply_modify(v) * (1.0 + self.final_pct) * self.final_mult
    }
}

// todo test

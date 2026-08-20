//! 聚合值
//! 
//! 多源公式合并，其修改器不变则值不变，保证了“一致性”

/// 聚合值，支持多维度修改
pub struct AggregateValue {
    /// 原始值，未经过修改器修改
    origin: f64,
    /// 当前值，经过修改器修改
    current: f64,
}

pub struct AggregateModifier {
    
}

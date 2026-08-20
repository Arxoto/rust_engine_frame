//! 基准锚点值
//! 
//! 只允许在基础值上进行修改，具备极佳的“修改可预测性”

/// 基准锚点值，具有修改效果可预测的特点，用作修改锚点
pub struct AnchorValue {
    /// 原始值，未经过修改器修改
    origin: f64,
    /// 当前值，经过修改器修改
    current: f64,
}

pub struct AnchorModifier {

}

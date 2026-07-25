//! 基于绝对时间戳实现的计时器
//! - 【缺点】无法获取进度
//! - 高性能，无需每帧更新，仅需只读比较即可，需要传入当前时间
//! - 适用于服务端验证、长期计时、数量规模庞大等场景

#[derive(Clone, Debug)]
pub struct StaticTimeline(pub f64);

#[derive(Clone, Debug)]
pub struct StaticTimer {
    /// 计时开始时间
    begin_at: f64,
    /// 计时结束时间
    end_at: f64,
}

// todo

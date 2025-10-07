//! 行为系统的抽象定义

/// 行为 纯逻辑 实现自定义复杂效果
///
/// 应该优先应用动作的物理和视觉效果
///
/// 需要支持不同实现存入同一个容器中（由于实例化的数量较小，不会有很大的性能损耗，选择方案二）
/// - 方案一：使用 enum 将多个行为放在一起，适用于极值性能的场景（手动维护）
/// - 方案二：使用 Box + dyn ，优点的易于扩展，缺点是性能稍差和基于堆内存
pub trait Behaviour<EnterParam, FrameParam, FrameEff, PhyParam, PhyEff> {
    fn will_enter(&self, p: &EnterParam) -> bool;

    fn on_enter(&mut self);
    fn on_exit(&mut self);

    fn tick_frame(&mut self, p: &FrameParam) -> FrameEff;
    fn process_physics(&mut self, p: &mut PhyParam) -> PhyEff;
}

/// 玩家操作 直接对应玩家意图
///
/// 该类型属性一般持久化存在，根据玩家输入实时修改
pub trait PlayerOperation {
    /// 指令是否应该激活对应操作
    fn op_active(&self) -> bool;
}

/// 玩家指令 从 [`PlayerOperation`] 中生成，用于传给状态机
///
/// 该类型属性生命周期在一帧内，每次根据玩家操作临时生成
#[derive(Clone, Default)]
pub struct PlayerInstruction<T: PlayerOperation>(pub(crate) T);

impl<T: PlayerOperation> PlayerOperation for PlayerInstruction<T> {
    fn op_active(&self) -> bool {
        self.0.op_active()
    }
}

impl<T: PlayerOperation> From<T> for PlayerInstruction<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

// ===========================
// impl
// ===========================

impl PlayerOperation for bool {
    fn op_active(&self) -> bool {
        *self
    }
}

impl<T: PlayerOperation + Clone> PlayerOperation for Option<T> {
    fn op_active(&self) -> bool {
        self.clone().is_some_and(|o| o.op_active())
    }
}

/// 对于 f64 容错值一般使用 1e-9 （防止加减乘除累积误差） 但是这里用作输入 没必要那么精确
const DEAD_ZONE: f64 = 1e-4;

// for direction or angle
impl PlayerOperation for f64 {
    fn op_active(&self) -> bool {
        self.abs() > DEAD_ZONE
    }
}

// ===========================
// test
// ===========================

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_bool() {
        assert!(true.op_active());
        assert!(!false.op_active());

        let o = true;
        assert!(o.op_active());
    }

    #[test]
    fn test_option() {
        assert!(!None::<bool>.op_active());
        assert!(!Some(false).op_active());
        assert!(Some(true).op_active());

        let o = Some(true);
        assert!(o.op_active());
    }

    #[test]
    fn test_f64() {
        assert!(!0.0000_0000_1.op_active());
        assert!(0.1.op_active());

        let o = 0.1;
        assert!(o.op_active());
    }
}

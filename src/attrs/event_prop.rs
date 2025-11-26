//! 动态属性 property 的响应事件类型
//!
//! 设计哲学：将所有所需的事件都通过返回值返回，外部系统自己去判断，保证框架的纯粹性
//!
//! 若想实现事件触发型效果（如：当占比小于【一定比率】时，自动修改当前值，用于斩杀或保命）
//! - 外部系统实现，在计算伤害之后，发送一个事件（游戏框架一般自带事件系统），在事件中增加例外处理

use crate::{cores::unify_type::FixedName, effects::native_effect::Effect};

#[derive(Debug, Default)]
pub struct DynPropAlterResult {
    /// 效果值 在扣血等场景下 效果值可能大于实际修改值
    pub value: f64, // pub-external
    /// 效果实际修改值
    pub delta: f64, // pub-external
}

impl DynPropAlterResult {
    /// 有益的
    pub fn is_beneficial(&self) -> bool {
        self.delta > 0.0
    }

    /// 有害的
    pub fn is_harmful(&self) -> bool {
        self.delta < 0.0
    }
}

pub struct DynPropProcessResult<S: FixedName> {
    /// 被哪个效果作用后达到最小值
    pub to_min_by: Option<Effect<S>>, // pub-external
}

use crate::base_lib::{
    cores::unify_types::FixedName,
    eff_attr_prop::upsert_container::{Upsert, UpsertContainer},
};

/// property 属性 一般用作角色资源槽 可被效果影响
#[derive(Debug, Default)]
pub struct Prop {
    current: f64,
}

impl Prop {
    pub fn new(current: f64) -> Self {
        Self { current }
    }

    /// 风格是把所有修改存入 buffer 然后一把梭哈
    pub fn refresh_value<S: FixedName, Timer: Upsert>(&mut self, buffer: &UpsertContainer<Timer>) {
        // 流程是先刷新上下限
        // 然后赋值当前值作为基准
        // 接着遍历 buffer 生效并记录来源（跟踪谁是凶手）
        // 基于上下限做对应调整
        // 若判定死亡则返回凶手
        let _ = buffer;
        todo!()
    }
}

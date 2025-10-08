//! 提示：为尽量保持项目纯净，使用命令检查是否有属性直接暴露给外部 `grep -r 'pub ' . | grep -v 'pub mod' | grep -v 'pub fn' | grep -v 'pub struct' | grep -v 'pub enum' | grep -v 'pub trait' | grep -v 'pub type' | grep -v '// pub-external'`

pub mod cores;

// Ability System Component
pub mod attrs;
pub mod effects;

// Damage System Component
pub mod damage;

// Motion System Component
pub mod motions;

// todo 修改的代码格式化
// todo `cargo test`

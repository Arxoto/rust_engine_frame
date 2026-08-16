# animations.rs 孤儿文件处理

Type: task
Status: ready-for-agent

`base_lib/motions/animations.rs` 仅含模块文档("动画播放器结束信号自动衔接下一段动画"),未被 `motions.rs` 声明(`motions.rs:25-31`),是死重。并入 `motions.rs`(保留该设计为模块文档)或删除;并同步更新 CLAUDE.md 对应条目(`animations.rs` 段落)。spec:`interface-deepening/spec.md`。

## 验收

- 选择其一:并入后 `motions.rs` 声明它且内容有效;或文件删除。
- CLAUDE.md 中关于 `animations.rs` 的段落与实际一致。
- `cargo build` 与 `cargo test --lib` 不受影响。

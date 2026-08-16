# Bevy 集成测试驱动真实框架逻辑

Type: task
Status: ready-for-agent

当前 `tests/bevy_tests.rs` + `tests/bevy_plugins/` 是 hello-world 样板,未驱动任何框架逻辑,"无游戏引擎依赖"承诺(README 项目介绍)未被验证。新增一份把 `base_lib` 真实逻辑(如属性效果/计时器)挂到 Bevy System 上的集成测试,证明同一套 `base_lib` 逻辑在 Bevy 下可运行。spec:`interface-demonstration-tests/spec.md`。

## 验收

- 测试在 `--no-default-features --features bevyproj,time_type_f64` 下通过。
- 测试内实际调用 `base_lib` 的公开接口(非空实现),并断言 Bevy World 中的结果。
- 与 Godot 侧共用同一 `base_lib` 源码,验证引擎无关性。

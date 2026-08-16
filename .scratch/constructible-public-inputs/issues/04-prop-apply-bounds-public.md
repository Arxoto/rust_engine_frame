# Prop::apply_bounds 公开化(文档承诺的工作流可执行)

Type: task
Status: resolved

`base_lib/eff_attr_prop/props.rs` 第 82 行文档指导上层执行 `refresh_bounds` → `apply_bounds`,但 `apply_bounds`(第 47 行)是私有方法,承诺的公开序列不可达。将 `apply_bounds`(或等价的公开 re-clamp 方法)提升为公开,并修正文档使序列可执行。spec:`constructible-public-inputs/spec.md`。

## 验收

- 上层可公开调用"刷新上下限 → 应用钳制"的工作流。
- 测试演示:提升上限再刷新后 `current` 被重新钳制;效果过期后 `current` 回落到边界。
- 文档与公开接口一致,不再指向私有方法。

## Answer

已实现：
- `Prop::apply_bounds` 提升为 `pub`,更新文档使其成为 `refresh_bounds` 之后的公开钳制入口。
- 两个测试：刷新上下限后 `apply_bounds` 将 `current` 钳到上限；低于下限时钳到下限。

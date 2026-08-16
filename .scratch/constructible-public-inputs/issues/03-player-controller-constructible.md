# PlayerCharacterController 公开构造器与只读访问

Type: task
Status: resolved

`base_lib/motions/player_controller.rs` 的 `PlayerCharacterController`(第 30-45 行)字段全私有、无构造器、无 getter,唯一方法是 `update()`(第 48-64 行),上层无法实例化或观察。提供基于 `PlayerInput` 的公开构造器与必要的只读访问(或等价可演示的公开方法)。spec:`constructible-public-inputs/spec.md`。

## 验收

- 上层可构造 `PlayerCharacterController` 并调用 `update()`。
- 至少一个测试演示构造 → 输入 → update → 观察输出(对照 `PlayerInput` 的公开字段)。
- 保留 `PlayerInput` 现有的公开字段设计。

## Answer

已实现：
- `PlayerCharacterController::new(player_input, input_buffer_window)` 公开构造器（缓冲时长作为参数，遵循 behaviours.rs 的时长参数约定）。
- 8 个只读访问（attack/block/jump/dodge 的 just_down 与 hold_down）。
- 测试：构造 → update → 观察 hold 状态随输入变化。

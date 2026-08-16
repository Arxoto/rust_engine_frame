# ActionData 公开构造路径

Type: task
Status: resolved

为 `base_lib/motions/actions.rs` 的 `ActionData`(第 33-56 行,`id`/`priority`/`order`/`enter_confition`/`state_tags` 全部私有,无构造器)提供公开构造函数或 builder,使 `ActionSwitcher::register_action`(第 128 行)成为可用 API。注意 `enter_confition` 字段名疑似笔误(condition)。spec:`constructible-public-inputs/spec.md`。

## 验收

- 上层可构造 `ActionData`(覆盖全部字段),`register_action` 可被调用。
- 一个空 `ActionSwitcher` 注册动作后可实际切换到该动作。
- 至少一个测试演示注册 → 切换 → 读取当前动作/标签。
- 不改动作切换的优先级与标签逻辑。

## Answer

已实现：
- `ActionData::new(id, priority, enter_condition, state_tags)` 公开构造路径（`order` 注册时自动赋值）。
- 顺带修正字段名笔误 `enter_confition` → `enter_condition`。
- 两个测试：注册→切换→读取标签；优先级+标签条件共同决定切换。

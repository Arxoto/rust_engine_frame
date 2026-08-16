# 装备 → 属性效果链测试

Type: task
Status: resolved
Blocked by: constructible-public-inputs/01

测试 `common_impl/combats/equips.rs` 的 `EquipWeapon`/`EquipArmor::equip`/`take_off` 写入外赋属性效果后,经刷新链(`attr_systems` 组合)反映到 `Attr::current` 的完整链路。当前装备写入容器但无读取方,装备无可见效果。spec:`interface-demonstration-tests/spec.md`。

## 验收

- 演示:equip → 刷新 → 对应外赋属性(锋利/质量/坚韧/柔韧)生效;take_off → 刷新 → 属性回落。
- 需要 `StaticTimeline` 驱动(可组合 `attr_systems` 或本测试内建时间线)。
- 为等级 C 的 upsert 语义统一提供调用侧样例。

## Answer

已实现（commit 45d3402，内联 `equips.rs`）：
- `equip_weapon_chain_reflects_to_attr` / `equip_armor_chain_reflects_to_attr`:equip → 容器写入外赋属性效果 → `try_refresh_dirty_attr` → `Attr::current` 生效;take_off → 刷新 → 属性回落。
- 测试注释按 spec 表述:本例仅含 inf 装备效果故单测无需时间线,但业务上其他有限时长效果仍需时间线 + `clean_expired_element`。
- 为等级 C 的 upsert 语义统一提供调用侧样例（`equips.rs:138-140` 的 `update_logic` 闭包）。

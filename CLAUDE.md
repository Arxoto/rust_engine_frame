# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test

```bash
# Build (default features = time_type_f64 + godotext: commonimpl + godot)
cargo build

# Build release (opt-level=3, codegen-units=1, lto=thin)
cargo build --release

# Build minimal (no engine binding; MUST also pick a time type, see Features)
cargo build --no-default-features --features baselib,time_type_f64

# Build core business logic only, no engine binding
cargo build --no-default-features --features commonimpl,time_type_f64

# Build with Bevy instead of Godot
cargo build --no-default-features --features bevyproj,time_type_f64

# Run all unit tests (default features)
cargo test --lib

# Run tests filtered by module path
cargo test --lib base_lib::motions::          # behaviours + controllers + ...
cargo test --lib base_lib::cores::design_patterns
cargo test --lib common_impl::combats::

# Run Bevy integration tests (requires the bevyproj feature)
cargo test --no-default-features --features bevyproj,time_type_f64

# Lint & format
cargo clippy
cargo fmt
```

**Gotcha:** every `--no-default-features` build must add `time_type_f64` (or `time_type_duration`) — the `time_type` module is gated behind those features and nothing compiles without one.

Note: on Windows, `.cargo/config.toml` sets `linker = "rust-lld.exe"` for the MSVC target (Bevy-recommended linker).

## Feature Flags

| Feature | Composition | Purpose |
|---------|-------------|---------|
| `time_type_f64` | (default) | `time_type::T = f64` time model (see below) |
| `time_type_duration` | (alternative) | `time_type::T = std::time::Duration` |
| `baselib` | (standalone) | Minimal dependency core, packable as a plain lib |
| `commonimpl` | `baselib` | Core business logic: eff_attr_prop, motions, combats |
| `godotext` | `commonimpl` + `godot` | Default. Godot GDExtension via godot-rust |
| `bevyproj` | `commonimpl` + `bevy` | Bevy engine integration (no godot binding) |

Default features = `["time_type_f64", "godotext"]`, which transitively enables `commonimpl` + `baselib` + `godot`.

**Time type:** `base_lib::cores::unify_types::time_type` centralizes the time model — `T` (f64 or Duration), `ZERO`, `INFINITY`, `DEFAULT_REFRESH_PERIOD` (5s, used by the upsert cleaner), `RESET_TIMELINE_PERIOD` (1 year, timeline-drift guard), and `to_f64()`. All timer code is written against `time_type::T`.

## Architecture

**Single active tree.** The mid-refactor "two parallel trees" state is over: the legacy object-oriented tree (top-level `attrs/`, `effects/`, `combats/`, `cores/`) was migrated into `base_lib`/`common_impl` and deleted (see git history: "迁移历史实现的战斗模块、属性效果模块、计时器模块"). There is only the ECS-oriented tree now. `base_lib` is always compiled; `common_impl` and `godot_ext_impl` are feature-gated.

### `src/base_lib/` (always compiled)

#### `base_lib/cores/` — foundational abstractions
- `unify_types.rs` — the trait `FixedName` (Eq+Hash+Clone+Debug), the generic ID/name bound everywhere. (The old `FixedString` trait was removed; engine string types are now adapted by wrapping into newtypes that implement `FixedName`, see `godot_ext_impl/adapter.rs`.) Also the `time_type` module described above.
- `design_patterns.rs` — **the central composition mechanism.** The design doc records why: `TickTimer` is self-sufficient while `StaticTimer` needs a `StaticTimeline`, and the code must not be duplicated for both. The chosen approach (**方案二**): traits declare their context via `DependCtx { type Ctx<'a>: Copy; }` as an associated type — `Ctx = ()` for self-sufficient types, `Ctx = &'a StaticTimeline` for timeline-dependent ones. `Union<T, U>` is the composition pair type used to build prefab proxies. `WithInto<Ctx>` (方案一, blanket-impl based) is kept as an alternative but the doc notes it hits old-trait-solver ambiguity and is **not** the default. Read this file before adding any timer behavior.
- `timers/` — trait hierarchy plus concrete builds:
  - `tiny_timer.rs` — the traits: `Tickable` (delta `tick`), `TimerProgress` (elapsed/remaining), `TimerView` (is_completed), `TimerControl` (reset), `TimerPauseView`/`TimerPauseControl` (pause/resume), `CyclicalTrigger` (try_trigger), `HasTimer` (exposes the embedded timer type). Most are bound on `DependCtx`. The module doc walks through the composition-approach trade-offs that led to `Union`.
  - `tick_timer.rs` — `TickTimer` (finite delta timer, the per-frame workhorse).
  - `tick_trigger.rs` — `InfiniteTickTrigger` (delta, infinite cycle).
  - `static_timer.rs` — `StaticTimeline` (wraps a `TickTimer` ticking to infinity as a reference clock) + `StaticTimer` (absolute timestamps, `DependCtx` = `&StaticTimeline`). No per-frame tick — cheap read-only comparisons; ideal for many long-lived effects. **Lifecycle:** the timeline must be reset periodically to avoid float drift — call `StaticTimeline::reset_timeline_and_get_diff()` and then `StaticTimer::fix_timeline_diff(diff)` on every dependent timer (see `attr_systems::try_reset_timeline`).
  - `static_trigger.rs` — `InfiniteStaticTrigger`, `FewShotStaticTrigger`.
  - `pause_prefab.rs` — `PausePrefab` (freeze tag, default unfrozen). Proxies any timer; intervenes in `tick` via `Union`.
  - `few_shot_times.rs` — `FewShotTimes(limit)` (finite-cycle tag). Intervenes in `TimerView`/`TimerControl`/`CyclicalTrigger` via `Union`; builder fns like `FewShotTimes::of_timer_view(&timer)` / `of_timer_control(&mut timer)` produce the composed view. Prefab traits delegate through `Union<&Tag, &Timer>`, never on a bare struct.
- `tiny_tags.rs` — `TinyTag<T>` logic expressions (`Always`/`Has`/`Not`/`And`/`Or`) evaluated against a `PureTagContainer`.

#### `base_lib/eff_attr_prop/` — attributes & effects (the ECS-flattened core)
The module doc (`eff_attr_prop.rs`) records the design decision: attributes and effect-containers are **flattened components**, effects are plain structs held by containers (not entities), and timers live inside effects. Rationale: cache-friendly, cohesion, no per-effect entity churn.
- `effects.rs` — `Effect<S>` pure data (`from_name`, `effect_name`, `effect_value`) + `EffectMeaning` trait / `EffectMean` enum (good/bad/neutral).
- `attr_eff.rs` — `AttrEffect<S, Timer>` typed by `AttrEffectType` (BasicAdd/BasicPer/FinalPer/FinalMul) and `AttrModifier` with the formula `(basic_add + origin * basic_per) * final_per * final_mul`. Implements `Upsert`.
- `attrs.rs` — `Attr`: single value (`origin` + `current`), recomputed from an iterator of `AttrEffect` via `AttrModifier`.
- `prop_bounds_eff.rs` — `PropBoundsEffect<S, Timer>` (modifies upper/lower bounds; wraps an `AttrEffect` restricted to BasicAdd/BasicPer) with `PropBoundsEffectType` (UpperAdd/UpperPer/LowerAdd) and `PropBoundsEffectTarget` (Upper/Lower).
- `props.rs` — `Prop`: resource pool (`upper`/`lower` `Attr` bounds + `current`), `refresh_bounds`/`apply_bounds`, and instant value change via `apply_eff`/`apply_eff_checked` (returns `PropAlterResult { real_eff_val }`). Note: the old `PropEffect` type was removed — instant changes (damage/heal) now go directly through `Prop::apply_eff`, and death-cause tracking lives in `common_impl/combats/damages.rs` (`DamageInfo`).
- `upsert_container.rs` — `UpsertContainer<E: Upsert>`: `Vec<Option<E>>` effect list with **upsert-by-id** (id = `(effect_name, from_name)`), hole-counting compaction (`try_clean_hole`, threshold 25% or >50 holes), and a dirty flag. `UpsertContainerCleaner` triggers periodic hole cleanup (default `time_type::DEFAULT_REFRESH_PERIOD`, 5s). Docs note: array storage beats `FxHashMap` up to ~20–50 elements.
- `attr_systems.rs` — the ECS-style **System** layer, split into generic functions (not one monolithic `process_tick`): `clean_expired_element` (generic via `HasTimer`), `try_refresh_dirty_attr`, `try_clean_hole`, `try_reset_timeline`. They are free generic functions meant to be composed per-entity by the caller — the module's test (`example_process_tick`) shows an OO-style composition. (combats does not yet invoke them.)

#### `base_lib/motions/` — player movement, action switching, input
- `actions.rs` — **tag-based action switching** (`ActionSwitcher`), not a classic state machine. `ActionData` (id, priority, order, `TinyTag` enter condition, state_tags). On each `switch_next_action`, the highest-priority action whose enter condition matches the current tag set wins; tags are timed or infinite. Design rationale + reference (ACT Game Action System / UE GAS GameplayTag) in the module doc.
- `controllers.rs` — input translation: `InputOperation` (raw), then instruction types `InstructionStrictJustOn` (strict previous-frame edge), `InstructionBufferedJustOn` (pre-input/input-buffer window via `TickTimer`), `InstructionStateOn` (hold), `InstructionStillKeep` (sealed via a private `Sealed` trait). Traits `ActiveInput`/`AbstractInstruction`.
- `player_controller.rs` — `PlayerInput` → `PlayerCharacterController` (sample 2D mapping: move/attack/block/jump/dodge).
- `behaviours.rs` — behaviour samples + helpers (`JumpBehaviourHelper` with `JumpStat` FSM + coyote time + higher jump, `LandingRoll`, `ReadyToJump`). The module doc records the **action/behaviour layering decision** (dependencies between behaviours instead of behaviour-level state machines) and the behaviour-effect design (pure functions returning side effects for the framework to aggregate).
- `animations.rs` — 【currently not part of the module tree】 the file exists on disk but `motions.rs` no longer declares it (it holds only a module doc, no code). Re-integrate it into `motions.rs` or delete it.
- `motions.rs` module doc — motion physics constants/derivations (run/jump/glide speeds, jump-height math) live here.

### `src/common_impl/combats/` — gated by `feature = "commonimpl"`

The **combat design/balance spec** lives across the module docs of `combats.rs` (气力/信念 stat design, energy/poise/shield systems, damage types & balancing constraints) and `damages.rs` (受伤上限/伤害成长 balance analysis + formulas) — read them even while implementing.

- `combat_inherents.rs` — 内禀属性 (character growth): `Strength` (气力) and `Belief` (信念), each a newtype around `Attr` plus its `UpsertContainer<AttrEffect<S, StaticTimer>>` effect container (`StrengthEffs`/`BeliefEffs`).
- `combat_additions.rs` — 外赋属性 (equipment bonuses): `WeaponSharp` (锋利), `WeaponMass` (质量), `ArmorHard` (坚韧), `ArmorSoft` (柔韧), `ArmorMass` (质量), same newtype+container shape.
- `combat_units.rs` — 战斗属性 (combat-time units): `Health`, `Stamina` (耐力/平衡), `Magicka` (能量/气势), and shields (`ShieldArcane`/`ShieldDefence`/`ShieldSubstitute`) as `Prop`-based components driven by `PropBoundsEffect` containers. Module doc carries a todo roadmap (initialization from inherents, shield generation, magicka cost `try_cost_magicka`, 削韧 `cut_stamina`).
- `equips.rs` — `EquipWeapon`/`EquipArmor`: equipment data that generates 外赋属性 effects.
- `damages.rs` — damage pipeline: `DamageEffectBuffer`/`DamageEffect`/`DamageInfo` (death-cause tracking) with `DamageType` (KarmaTruth 真实 / PhysicsShear 物理剪切 / PhysicsImpact 物理冲击 / MagickaArcane 魔法奥术 / BrokeShieldDefence 防护破盾 / BrokeShieldArcane 奥术破盾), `DamageCalc` (Val/CurPer/MaxPer), and `MagickaEnergyLevel`. The nested `damage_system` module carries the formulas and spec: `merge_damages` (每帧同类型伤害合并), `apply_damages`, `calc_damage_scale`, `calc_health_max`, `calc_magicka_value`/`calc_magicka_max`, `calc_defence_shield`.

The combat structs **deliberately use `pub` fields** so they can be split/flattened onto ECS entities — this is the documented exception to the minimize-`pub` convention.

### `src/godot_ext_impl/` — gated by `feature = "godotext"`

- `adapter.rs` — New Type pattern wrapping Godot `StringName` into `FixedNameWrapper` and `GString` into `FixedStringWrapper`, both implementing the crate's `FixedName` for engine interop (the old `FixedString` trait was removed).
- `attr_impl.rs` — empty stub.
- `godot_ext_impl.rs` — the module root hosts `ExSystemComponent`, the `#[gdextension]` entry point (see its `res://*.gdextension` setup notes).

## Key Conventions

- **`DependCtx`/`Union` composition** — implement new timer/behavior traits via the associated-type `DependCtx` bound, and build prefab tags as `Union` proxies with delegating impls. Don't invent a new wrapper pattern; read `base_lib/cores/design_patterns.rs` and `timers/pause_prefab.rs` first.
- **Minimize `pub` fields** — Prefer `pub(crate)` or accessor methods. Use `// pub-external` comment marker on fields that legitimately need to be public. Check violations:
  ```bash
  grep -r 'pub ' src/ | grep -v 'pub mod' | grep -v 'pub fn' | grep -v 'pub struct' | grep -v 'pub enum' | grep -v 'pub trait' | grep -v 'pub type' | grep -v '// pub-external'
  ```
  Exception: ECS component structs in `common_impl/combats/` use `pub` fields deliberately for entity flattening.
- **HashMap performance** — Use `rustc-hash` (`FxHashMap`) for game-critical paths. Small datasets (<~30 elements) may be faster with `Vec` linear search. Pre-allocate with `with_capacity(capacity.next_power_of_two())`; if the element count is known in advance, add a comment/marker explaining it. Check violations: `grep -r 'with_capacity' . | grep -v 'next_power_of_two'`
- **New Type pattern** — Wrap external types to implement crate traits (orphan rule workaround). See `godot_ext_impl/adapter.rs` and the tests in `base_lib/cores/unify_types.rs`.
- **Effect stacking is upsert, not accumulate** — `UpsertContainer` matches by `(effect_name, from_name)`; re-applying the same id overwrites the existing effect rather than stacking separate instances. Damage-source tracking is therefore not precise.
- **Timer tick ordering** — "help" timers (grace windows like coyote time) should tick *after* business logic; "limit" timers (cooldowns) tick *before*. Tests follow "先业务后 tick".
- **StaticTimer lifecycle** — a `StaticTimer` must be created against a `StaticTimeline` and its traits take the timeline as context. The timeline must be reset (with `fix_timeline_diff` on all dependents) once no live timer depends on it (see `attr_systems::try_reset_timeline`).
- **Comments & commits in Chinese** — code comments, module docs, and commit messages are written in Chinese.

## Testing

- Unit tests are inline in `#[cfg(test)]` blocks within each source file.
- Integration tests live in `tests/`; `tests/bevy_tests.rs` + `tests/bevy_plugins/` are gated on `feature = "bevyproj"` and must be run with `--no-default-features --features bevyproj,time_type_f64`. `tests/state_machine_action_tests.rs` is a non-gated smoke test that exercises the `tests/common` helper.
- Test helpers in `tests/common/common_helper.rs`.
- Code comments and commit messages are in Chinese.

## Agent skills

### Issue tracker

Issues and specs are tracked as markdown files under `.scratch/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default triage labels: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context — `CONTEXT.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.

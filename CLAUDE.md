# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test

```bash
# Build (default features = godotext: commonimpl + godot)
cargo build

# Build release (opt-level=3, codegen-units=1, lto=thin)
cargo build --release

# Build minimal (no godot, no bevy)
cargo build --no-default-features --features baselib

# Build core business logic only, no engine binding
cargo build --no-default-features --features commonimpl

# Build with Bevy instead of Godot
cargo build --no-default-features --features bevyproj

# Run all unit tests (default features)
cargo test --lib

# Run tests filtered by module path
cargo test --lib base_lib::motions::          # behaviours + controllers
cargo test --lib base_lib::cores::design_patterns
cargo test --lib attrs::dyn_prop              # legacy tree
cargo test --lib combats::combat_units        # legacy tree
cargo test --lib cores::static_timer          # legacy tree

# Run Bevy integration tests (requires the bevyproj feature)
cargo test --no-default-features --features bevyproj

# Lint & format
cargo clippy
cargo fmt
```

Note: on Windows, `.cargo/config.toml` sets `linker = "rust-lld.exe"` for the MSVC target (Bevy-recommended linker).

## Feature Flags

| Feature | Composition | Purpose |
|---------|-------------|---------|
| `baselib` | (standalone) | Minimal dependency core, packable as a plain lib |
| `commonimpl` | `baselib` | Core business logic: attrs/effects (`eff_attr_prop`), motions, combats |
| `godotext` | `commonimpl` + `godot` | Default. Godot GDExtension via godot-rust |
| `bevyproj` | `commonimpl` + `bevy` | Bevy engine integration (no godot binding) |

Default features = `["godotext"]`, which transitively enables `commonimpl` + `baselib` + `godot`.

## Architecture

**This crate is mid-refactor toward an ECS-compatible design** (see README: "重构，兼容 ECS 思想"). The result is **two parallel module trees** that must not be confused:

- **Active (new) tree** — `src/base_lib/` + `src/common_impl/`. Actively developed, ECS-oriented. Commit activity lives here.
- **Legacy (old) tree** — top-level `src/attrs/`, `src/effects/`, `src/combats/`, `src/cores/`, `src/godot_ext_impl/`. A self-contained older object-oriented implementation that has not been fully migrated/removed yet. Nothing in `base_lib`/`common_impl` references it and vice versa.

**Rule of thumb: write new code in `base_lib`/`common_impl`.** Only touch the legacy tree to migrate it.

### 1. Active tree: `src/base_lib/` (always compiled)

#### `base_lib/cores/` — foundational abstractions
- `unify_types.rs` — trait aliases `FixedName` (Eq+Hash+Clone+Debug) and `FixedString` (…+Default). Implemented for integer types, `&str`, `String`. Used as the generic ID/name bound throughout.
- `design_patterns.rs` — **the central composition mechanism**: `ContextWrapper<I, Ctx>` plus `WithContext`/`WithInto` traits. A bare type stores only its data; behavior traits are implemented on `ContextWrapper<&mut X, &mut Ctx>` (composition + delegation). `WithInto<Ctx, Target>` lets a generic handler take `(source, ctx)` and produce a `ContextWrapper`, enabling one generic function to serve many types. This is how timers get their context injected (e.g. `StaticTimer` is meaningless without a `StaticTimeline`, so its traits are only implemented on the wrapper).
- Timers — a trait hierarchy plus concrete builds:
  - `tiny_timer.rs` — traits: `TinyTimer` (progress readout), `TickTimer` (delta-based `tick`), `FlowingTimer[Readonly]` (finish/restart), `FreezableTimer[Readonly]`, `CyclicalTimer`. Also the composable **tags** `FreezableTickTag` and `FewShotCycleTag`, whose trait impls are delegated *through* `ContextWrapper` to the wrapped timer.
  - `tick_timer.rs` — concrete delta timers `TinyTickTimer` (finite) and `InfTickTrigger` (infinite trigger).
  - `tick_timer_builders.rs` — prebuilt compositions `FreezeTickTimer`, `FreezeInfTickTrigger`, `FreezeFewShotTickTrigger` (data) + trait impls delegating to tags.
  - `static_timer.rs` — `StaticTimer` + `StaticTimeline`: absolute-timestamp timers (no per-frame `tick`; cheap read-only comparisons). High-performance, suited to many long-lived effects. `StaticTimer::new(&timeline, duration)` captures `end_at` from the timeline's current time. The timeline is a `FreezeTickTimer` ticking to `f64::INFINITY`; it must be `restart_timeline()`d once no timer depends on it (to avoid float drift).
  - `tiny_tags.rs` — `TinyTag<T>` logic expressions (`Always`/`Has`/`Not`/`And`/`Or`) evaluated against a `TinyTagContainer`.

#### `base_lib/eff_attr_prop/` — attributes & effects (the ECS-flattened core)
The module doc (`eff_attr_prop.rs`) records the design decision: attributes and effect-containers are **flattened components**, effects are plain structs held by containers (not entities), and timers live inside effects. Rationale: cache-friendly, cohesion, no per-effect entity churn.
- `effects.rs` — `Effect<S>` pure data: `from_name`, `effect_name`, `effect_value` + `EffectMeaning` (good/bad/neutral).
- `attr_eff.rs` — `AttrEffect<S, Timer>` (typed by `AttrEffectType`: BasicAdd/BasicPer/FinalPer/FinalMul) and `AttrModifier` with the formula `(basic_add + origin * basic_per) * final_per * final_mul`. Implements `Upsert`.
- `attrs.rs` — `Attr`: single value (`origin` + `current`), recomputed from an iterator of `AttrEffect` via `AttrModifier`.
- `prop_eff.rs` / `prop_bounds_eff.rs` — `PropEffect<S>` (instant value change, e.g. damage/heal) and `PropBoundsEffect<S, Timer>` (modifies upper/lower bounds; itself an `AttrEffect` restricted to BasicAdd/BasicPer).
- `props.rs` — `Prop`: resource pool (`upper`/`lower` `Attr` bounds + `current`), `refresh_bounds`/`apply_effs`, returns `PropAlterResult` (first harmful source, `current_le_zero`).
- `upsert_container.rs` — `UpsertContainer<E: Upsert>`: `Vec<Option<E>>` effect list with **upsert-by-id** (id = `(effect_name, from_name)`), hole-counting compaction (`try_clean_hole`, threshold 25% or >50 holes), and a dirty flag. `UpsertContainerCleaner` triggers periodic hole cleanup (default 5s). Docs note: array storage beats `FxHashMap` up to ~20–50 elements.
- `attr_systems.rs` — the ECS-style **System**: `process_tick(delta, timeline, cleaner, &mut [(attr, effs)])` ages/expires effects, refreshes `Attr` values, does periodic hole cleanup, and restarts the timeline when all containers are empty. Also `clean_expired_element` (generic via `WithInto`). Marked `todo`: split into finer-grained systems.

#### `base_lib/motions/` — player movement, action switching, input
- `actions.rs` — **tag-based action switching** (`ActionSwitcher`), not a classic state machine. `ActionData` (id, priority, order, `TinyTag` enter condition, state_tags). On each `switch_next_action`, the highest-priority action whose enter condition matches the current tag set wins; tags are timed or infinite. Design rationale + reference (ACT Game Action System / UE GAS GameplayTag) in the module doc.
- `controllers.rs` — input translation: `InputOperation` (raw), then instruction types `InstructionStrictJustOn` (strict previous-frame edge), `InstructionBufferedJustOn` (pre-input/input-buffer window via `TinyTickTimer`), `InstructionStateOn` (hold), `InstructionStillKeep`. Traits `ActiveInput`/`AbstractInstruction`; `InstructionStillKeep` is sealed via a private `Sealed` trait.
- `player_controller.rs` — `PlayerInput` → `PlayerCharacterController` (sample 2D mapping: move/attack/block/jump/dodge).
- `behaviours.rs` — behaviour samples + helpers (`JumpBehaviourHelper` with `JumpStat` FSM + coyote time + higher jump, `LandingRoll`, `ReadyToJump`). The module doc records the **action/behaviour layering decision** (dependencies between behaviours instead of behaviour-level state machines) and the behaviour effect design (pure functions returning side effects for the framework to aggregate). Motion physics constants/derivations live in `base_lib/motions.rs`.

### 2. Active tree: `src/common_impl/combats/` — gated by `feature = "commonimpl"`

- `combat_inherents.rs` — `CombatInherentAttr`: 气力 strength, 信念 belief (each `Attr` + `UpsertContainer<AttrEffect<S, StaticTimer>>`), shared `StaticTimeline` + `UpsertContainerCleaner`, and a `process_tick` that forwards to `attr_systems::process_tick`.
- `combat_additions.rs` — `CombatAdditionAttr`: weapon (锋利 sharp / 质量 mass) and armor (坚韧 hard / 柔韧 soft / 质量 mass) attributes, same shape.

Both structs deliberately use **`pub` fields** so they can be split/flattened onto ECS entities — this is the documented exception to the minimize-`pub` convention.

### 3. Legacy tree — top-level modules (not yet migrated)

Last substantially changed Feb 2026 (top-level `cores/` doc comment update May 2026). Forms its own dependency chain: `combats` → `attrs` + `effects` + `cores`; `attrs` → `cores`. The only cross-tree reference is `godot_ext_impl/adapter.rs` → `cores::unify_type`.
- `src/attrs/` — `DynAttr`, `DynProp` (resource pool with instant/duration/periodic effects), `EffectContainer`, `event_prop`. The older, heavier per-effect-processing model.
- `src/effects/` — `Effect`, `native_duration`, `duration_effect`.
- `src/combats/` — `CombatUnit`, `CombatHealthShield`, `DamageType`, `NumericalBalancer`. **The module doc in `combats.rs` holds the full combat design/balance spec** (气力/信念, energy/poise, shields, damage formulas) — read it even while the implementation is being migrated.
- `src/cores/` — older copies of `unify_type`, `static_timer`, `tiny_timer`.
- `src/godot_ext_impl/` — skeletal: `adapter.rs` wraps Godot `StringName`/`GString` into `FixedNameWrapper`/`FixedStringWrapper` (New Type pattern); `attr_impl.rs` is empty; `ExSystemComponent` is the `#[gdextension]` entry point.

## Key Conventions

- **`ContextWrapper`/`WithInto` composition** — implement new timer/behavior traits on `ContextWrapper<&mut X, &mut Ctx>` (or the composed builder struct), not on the bare data type. Read `base_lib/cores/design_patterns.rs` before adding any timer behavior.
- **Minimize `pub` fields** — Prefer `pub(crate)` or accessor methods. Use `// pub-external` comment marker on fields that legitimately need to be public. Check violations:
  ```bash
  grep -r 'pub ' src/ | grep -v 'pub mod' | grep -v 'pub fn' | grep -v 'pub struct' | grep -v 'pub enum' | grep -v 'pub trait' | grep -v 'pub type' | grep -v '// pub-external'
  ```
  Exception: ECS component structs in `common_impl/combats/` use `pub` fields deliberately for entity flattening.
- **HashMap performance** — Use `rustc-hash` (`FxHashMap`) for game-critical paths. Small datasets (<~30 elements) may be faster with `Vec` linear search. Pre-allocate with `with_capacity(capacity.next_power_of_two())`. Check violations: `grep -r 'with_capacity' . | grep -v 'next_power_of_two' | grep -v EVENT_LIST_CAPACITY`
- **New Type pattern** — Wrap external types to implement crate traits (orphan rule workaround). See `godot_ext_impl/adapter.rs` and the tests in `base_lib/cores/unify_types.rs`.
- **Effect stacking is upsert, not accumulate** — `UpsertContainer` matches by `(effect_name, from_name)`; re-applying the same id overwrites the existing effect rather than stacking separate instances. Damage-source tracking is therefore not precise.
- **Timer tick ordering** — "help" timers (grace windows like coyote time) should tick *after* business logic; "limit" timers (cooldowns) tick *before*. Tests follow "先业务后 tick".
- **StaticTimer lifecycle** — a `StaticTimer` must be created against a `StaticTimeline` and its traits only exist on the `ContextWrapper`. The timeline must be restarted once no live timer depends on it (see `attr_systems::process_tick`).

## Testing

- Unit tests are inline in `#[cfg(test)]` blocks within each source file (both trees).
- Integration tests live in `tests/`; `tests/bevy_tests.rs` + `tests/bevy_plugins/` are gated on `feature = "bevyproj"` and must be run with `--no-default-features --features bevyproj`.
- Test helpers in `tests/common/common_helper.rs`.
- Code comments and commit messages are in Chinese.

## Agent skills

### Issue tracker

Issues and specs are tracked as markdown files under `.scratch/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default triage labels: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context — `CONTEXT.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test

```bash
# Build (default features includes commonimpl + godot)
cargo build

# Build release (opt-level=3, lto=true, codegen-units=1)
cargo build --release

# Build minimal (no godot binding, no combat system)
cargo build --no-default-features --features baselib

# Build with only business logic, no godot
cargo build --no-default-features --features commonimpl

# Run all tests
cargo test

# Run tests for a specific module
cargo test --lib attrs::dyn_prop
cargo test --lib combats::combat_units
cargo test --lib motions::abstracts::action

# Lint & format
cargo clippy
cargo fmt
```

## Feature Flags

| Feature | Composition | Purpose |
|---------|-------------|---------|
| `baselib` | (standalone) | Minimal dependency core, packable as a plain lib |
| `commonimpl` | (standalone) | Core business logic: attrs, effects, motions |
| `godotext` | `commonimpl` + `godot` | Godot GDExtension bindings via godot-rust, includes combat system |

Default features = `["godotext"]`, which transitively enables `commonimpl` and `godot` crate.

## Architecture

The crate is a **game engine framework** organized into four layered systems:

### 1. Core (`src/cores/`)
- `unify_type.rs` — Trait aliases `FixedName` (Eq+Hash+Clone+Debug) and `FixedString` (Eq+Hash+Clone+Debug+Default) used as generic bounds throughout. Both `String`/`&str` and numeric types implement them, allowing IDs to be strings or integers.
- `tiny_timer.rs` — Minimal timer with start/stop/flow control used for cooldowns and pre-input windows.

### 2. Ability System (`src/attrs/` + `src/effects/`)
- **Effects** (`effects/`): `Effect<S>` is the atomic unit (name, source, value). `EffectBuilder` constructs instant/infinite/duration effects. `ProxyEffect` and `ProxyDuration` traits provide composable accessor delegation.
- **Duration effects** (`duration_effect.rs`): `(Effect, Duration)` tuple with `ProxyDurationEffect` trait — the standard "persistent effect" type used everywhere.
- **Attributes** (`attrs/`):
  - `DynAttr<S>` — Single attribute (origin + current) modified by stacked effects with additive/percent/final_percent layers. Effects auto-expire via `process_time`.
  - `DynProp<S>` — Resource pool (current clamped between min and max), each bound being a `DynAttr`. Supports three effect kinds: **instant** (one-shot), **duration** (persistent, modifies min/max bounds), **periodic** (DOT/HoT/regen). This is the primary type for health, shields, stamina, energy, entropy.
  - `EffectContainer<S, E>` — Ordered effect list using `Vec<Option<E>>` with in-place compaction. Favors performance over memory (never shrinks capacity unless explicitly refreshed).

### 3. Combat System (`src/combats/`) — gated by `feature = "commonimpl"`
- `CombatUnit<S>` — Full combat entity bundling health+shields, energy, stamina (poise/balance), element bars (entropy, electric), plus inherent (strength/belief) and addition (weapon/armor) attributes.
- `CombatHealthShield` — Layered defense: health → substitute shield → defense shield → arcane shield. Damage types pierce different layers in different orders.
- `DamageType` enum — KarmaTruth, PhysicsShear, PhysicsImpact, MagickaArcane + BrokeShield variants.
- `NumericalBalancer` — Centralized combat math (damage scaling, health max, energy levels, defense shield calc).

### 4. Motion/State Machine System (`src/motions/`)
- **Abstracts** (`abstracts/`):
  - `Action<S, Event, ExitParam, ExitLogic, PhyEff>` — Data-driven action: enter/exit events, logic-based exits (for combos), animation chains, per-animation physics effects. Priority-based switch rules with per-action overrides.
  - `Behaviour` trait — Logic-driven behavior (Box+dyn), handles `will_enter`/`on_enter`/`on_exit`/`tick_frame`/`process_physics`.
- **State machines**: `PlayerMachine<S>` composes `ActionMachine` and `BehaviourMachine`. Runs `tick_frame` (render) then `process_physics` in order: action first, then behaviour, `EffGenerator` merges outputs.
- **Motion modes**: OnFloor, InAir, UnderWater, ClimbWall, FreeStat, Motionless. Each mode has a corresponding behaviour in `motion_behaviours/`.
- **Player controller pipeline** (`player_controller.rs`):
  - `PlayerController` (raw input per frame) → `PlayerOperationCollection` (client-side, with pre-input state) → `PlayerInstructionCollectionRaw` (client→server transport) → `PlayerInstructionCollectionFinal` (server→client echo). The `once`/`keep`/`hold` tri-state system enables pre-input buffering.

### 5. Godot Extension (`src/godot_ext_impl/`) — gated by `feature = "godotext"`
- Wraps Godot's `StringName`/`GString` into `FixedNameWrapper`/`FixedStringWrapper` to satisfy the trait aliases. Extension entry point uses `#[gdextension]`.

## Key Conventions

- **Minimize `pub` fields** — Prefer `pub(crate)` or accessor methods. Use `// pub-external` comment marker on fields that legitimately need to be public. Check violations with:
  ```bash
  grep -r 'pub ' src/ | grep -v 'pub mod' | grep -v 'pub fn' | grep -v 'pub struct' | grep -v 'pub enum' | grep -v 'pub trait' | grep -v 'pub type' | grep -v '// pub-external'
  ```
- **HashMap performance** — Use `rustc-hash` (`FxHashMap`) for game-critical paths. Small datasets (<~30 elements) may be faster with `Vec` linear search. Pre-allocate with `with_capacity(next_power_of_two())`.
- **New Type pattern** — Wrap external types to implement crate traits (orphan rule workaround). See `godot_ext_impl/adapter.rs` and the test in `cores/unify_type.rs`.
- **Effect system is non-idempotent** — Stacking effects by name overwrites the source rather than accumulating separate instances. This simplifies management but means damage-source tracking isn't precise.

## Testing

- Unit tests are inline in `#[cfg(test)]` blocks within each source file.
- Integration tests live in `tests/` (currently sparse — `state_machine_action_tests.rs` is a placeholder example).
- Test helpers in `tests/common/common_helper.rs`.

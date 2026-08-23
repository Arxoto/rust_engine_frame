# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Intro

The project aims to be an **engine-agnostic general-purpose game foundation framework**: game business logic is driven by upper layers calling `pub fn`s exposed by this crate.

Because this is a library, some methods have no in-tree callers — each one is instead demonstrated by a doc test or an inline unit test showing how to use it.

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

## Agent skills

### Issue tracker

Issues and specs are tracked as markdown files under `.scratch/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default triage labels: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context — `CONTEXT.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.

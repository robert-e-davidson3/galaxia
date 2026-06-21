# Tech stack

The crates Galaxia depends on and why. Versions are authoritative in `Cargo.toml`; this file explains the roles. Update it when a dependency is added, dropped, or bumped across a meaningful boundary.

## Engine + rendering

- **`bevy` 0.14.1** — the game engine (ECS, rendering, input, audio, windowing). Everything is built around its `Startup` / `Update` / `FixedUpdate` schedules.
- **`bevy_prototype_lyon` 0.12.0** — 2D vector graphics (shapes drawn in-engine).
- **`bevy_ecs_tilemap` 0.14.0** — efficient tilemap rendering.
- **`bevy_framepace` 0.17.1** — frame-rate limiting / pacing.

## Physics

- **`bevy_rapier2d` 0.27.0** — Bevy integration for the Rapier 2D physics engine; drives item movement and collision.
- **`rapier2d` 0.22.0** — the underlying physics engine (used directly where the Bevy wrapper isn't enough).

## Data structures

- **`array2d` 0.3.2**, **`grid` 0.14.0**, **`ndarray` 0.16.1** — 2D grids and n-dimensional arrays for game state (boards, terrain, etc.).
- **`int-enum` 1.1.2** — integer ↔ enum conversions.
- **`once_cell` 1.20.2** — lazily-initialized statics.

## Procedural generation / randomness

- **`wyrand` 0.2.1** — fast PRNG; backs `src/libs/random.rs` (deterministic random).
- **`libnoise` 1.1.2**, **`perlin_noise` 1.0.1** — noise functions for procedural terrain.

## Images

- **`image` 0.25.5**, **`imageproc` 0.25.0** — image loading and processing.

## Serialization

- **`serde` 1.0.210** + **`serde_json` 1.0.128** — serialization, intended for save/load functionality.

## Build profile

`Cargo.toml` sets `opt-level = 3` for **both** `dev` and `release` — Bevy is unplayable unoptimized, so debug builds pay the compile cost to stay runnable. See `references/local-dev.md`.

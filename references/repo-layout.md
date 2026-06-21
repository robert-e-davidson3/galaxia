# Repo layout

How Galaxia's source is organized and how the pieces fit. This is the settled map of the codebase; when the structure changes, update this file.

Galaxia is a Rust game built on the [Bevy](https://bevyengine.org) engine: a collection of interconnected minigames bound into one overarching world. It's a vehicle for learning Rust and game design, so favor clarity over cleverness.

## Top level

- `src/` — all game code (see below).
- `assets/` — sprites, audio, and other runtime assets loaded by Bevy.
- `Cargo.toml` / `Cargo.lock` — crate manifest and lockfile. Note the dev profile uses `opt-level = 3` (Bevy is unusably slow unoptimized); see `references/tech-stack.md`.
- `flake.nix` / `flake.lock` — Nix dev shell (`references/local-dev.md`).
- `rustfmt.toml` — formatting config (`references/code-style.md`).

## `src/` — two main modules

- **`src/entities/`** — game entities and minigame implementations.
- **`src/libs/`** — utility libraries and cross-cutting systems.

`main.rs` wires everything together: it registers systems for Bevy's `Startup`, `Update`, and `FixedUpdate` schedules.

## Key systems

1. **Minigame system** (`src/entities/minigame.rs`) — the spine of the game.
   - Central `Minigame` enum holding every minigame variant.
   - Lifecycle management: spawn, levelup, item ingestion.
   - A prerequisites system that unlocks new minigames when others reach a level — `setup_minigame_unlocks()`.
   - A common interface every variant implements (name, description, area, level, …).

2. **Entity Component System** — built on Bevy's ECS.
   - Systems registered in `main.rs` across `Startup` / `Update` / `FixedUpdate`.
   - Physics via Rapier2D (`bevy_rapier2d`).

3. **Minigames** (`src/entities/minigames/`) — one module per minigame (button, rune, primordial_ocean, tree, ball_breaker, foundry, life, land, battery, chest, …). Each follows the same interface and can be gated behind prerequisites. To add one, follow `skills/add-minigame.md`.

## Core libraries (`src/libs/`)

- **`camera.rs`** — camera controls: zoom and player following.
- **`inventory.rs`** — item management and the inventory UI.
- **`mouse.rs`** — mouse input handling and hover text.
- **`collision.rs`** — collision detection utilities.
- **`random.rs`** — deterministic random number generation.
- **`area.rs`** — spatial area definitions (rectangular, circular).

## Game flow

1. The player spawns into a world with an initial set of minigames available.
2. Minigames are engaged by clicking their engage buttons.
3. Items are collected and fed to minigames.
4. Minigames level up when their conditions are met.
5. Leveling up unlocks new minigames based on prerequisites.
6. Item movement and collision are physics-based.

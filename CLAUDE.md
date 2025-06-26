# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Galaxia is a Rust-based game built with the Bevy engine featuring multiple interconnected minigames. The project is designed for learning Rust and game design through a collection of mini-games bound together in an overarching game world.

## Development Environment

This project uses Nix for reproducible development environments. The development shell includes:
- Rust stable with rustfmt and clippy
- Bevy game engine dependencies (graphics, audio, input)
- System libraries for Linux/Wayland/X11 support

### Common Commands

```bash
# Enter development environment
nix develop

# Build the project
cargo build

# Run the game
cargo run

# Run with optimizations (dev profile uses opt-level = 3)
cargo run --release

# Format code (max_width = 80 as per rustfmt.toml)
cargo fmt

# Run linting
cargo clippy

# Check for compilation errors without building
cargo check
```

## Architecture

### Core Structure

The codebase is organized into two main modules:

- **`src/entities/`**: Game entities and minigame implementations
- **`src/libs/`**: Utility libraries and systems

### Key Systems

1. **Minigame System** (`src/entities/minigame.rs`):
   - Central enum `Minigame` containing all minigame variants
   - Minigame lifecycle management (spawn, levelup, item ingestion)
   - Prerequisites system for unlocking new minigames
   - Common interface for all minigames (name, description, area, level)

2. **Entity Component System**:
   - Uses Bevy's ECS architecture
   - Systems are registered in `main.rs` for Startup, Update, and FixedUpdate
   - Physics integration with Rapier2D

3. **Minigames** (`src/entities/minigames/`):
   - Each minigame is a separate module with consistent interface
   - Examples: button, rune, primordial_ocean, tree, ball_breaker, etc.
   - Minigames can be unlocked based on prerequisites (other minigames reaching certain levels)

### Core Libraries (`src/libs/`)

- **`camera.rs`**: Camera controls with zoom and player following
- **`inventory.rs`**: Item management and inventory UI
- **`mouse.rs`**: Mouse input handling and hover text
- **`collision.rs`**: Collision detection utilities
- **`random.rs`**: Deterministic random number generation
- **`area.rs`**: Spatial area definitions (rectangular, circular)

### Game Flow

1. Player spawns in a world with initial minigames available
2. Minigames can be engaged by clicking their engage buttons
3. Items can be collected and fed to minigames
4. Minigames level up when conditions are met
5. Leveling up unlocks new minigames based on prerequisites
6. Physics-based item movement and collision detection

## Adding New Minigames

When adding a new minigame:

1. Create new module in `src/entities/minigames/`
2. Implement the standard interface (name, description, area, level, spawn, ingest_item, etc.)
3. Add variant to `Minigame` enum in `src/entities/minigame.rs`
4. Update all match statements in `minigame.rs`
5. Add to `setup_minigame_unlocks()` with appropriate prerequisites
6. Register update systems in `main.rs` if needed

## Key Dependencies

- **Bevy 0.14.1**: Game engine
- **bevy_rapier2d 0.27.0**: 2D physics
- **bevy_prototype_lyon 0.12.0**: 2D vector graphics
- **bevy_framepace 0.17.1**: Frame rate limiting
- **array2d, grid, ndarray**: Data structures for game grids
- **serde**: Serialization (for save/load functionality)

## Code Style

- Maximum line width: 80 characters (rustfmt.toml)
- Standard Rust naming conventions
- Bevy ECS patterns with Systems, Components, Resources
- Each minigame follows consistent interface pattern
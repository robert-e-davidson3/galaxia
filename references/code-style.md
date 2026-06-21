# Code style

Conventions for Galaxia's Rust. Keep new code consistent with what's here.

- **Line width: 80 columns max** — enforced by `rustfmt.toml` (`max_width = 80`). Run `cargo fmt` before considering work done.
- **Standard Rust naming conventions** — `snake_case` for functions/locals, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for consts.
- **Bevy ECS patterns** — model game state as Systems, Components, and Resources; prefer small focused systems registered in `main.rs` over monoliths.
- **Consistent minigame interface** — every minigame implements the same shape (name, description, area, level, spawn, ingest_item, …). When adding one, mirror an existing minigame rather than inventing a new shape; follow `skills/add-minigame.md`.
- **Clippy clean** — run `cargo clippy` and address warnings.

This is a learning project: favor readable, idiomatic code over clever code.

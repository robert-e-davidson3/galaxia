# Local dev

How to build, run, and check Galaxia locally. Kept current; if a command changes, fix it here.

## Environment

This project uses Nix for a reproducible dev environment. The dev shell (`flake.nix`) provides:

- Rust stable with `rustfmt` and `clippy`.
- Bevy's native dependencies (graphics, audio, input).
- System libraries for Linux / Wayland / X11.

Enter it with:

```bash
nix develop
```

The flake is the supported path on NixOS; a non-Nix machine needs a Rust toolchain plus Bevy's system deps installed by hand.

## Common commands

```bash
# Build the project
cargo build

# Run the game
cargo run

# Run with the release profile
cargo run --release

# Format (max_width = 80, per rustfmt.toml)
cargo fmt

# Lint
cargo clippy

# Typecheck without producing a binary
cargo check
```

## Notes

- **Both `dev` and `release` profiles set `opt-level = 3`** (`Cargo.toml`). Bevy is too slow to play unoptimized, so even debug builds are optimized — expect longer compiles. See `references/tech-stack.md`.
- `cargo fmt` enforces the 80-column width from `rustfmt.toml`; see `references/code-style.md`.

# Skill: add a minigame

**When:** adding a new minigame to Galaxia. Run this start to finish; it can be done cold.

Every minigame is a module under `src/entities/minigames/` that implements the same interface and is registered through the central `Minigame` enum in `src/entities/minigame.rs`. The work is mostly: write the module, then wire it into the enum and the match statements that fan out over it. The job isn't done until `cargo build && cargo clippy && cargo fmt` are all clean. Mirror an existing minigame (e.g. `button.rs` for something simple, `ball_breaker.rs` for something with internal state) rather than inventing a new shape — see `references/repo-layout.md` and `references/code-style.md`.

## Steps

1. **Create the module** — `src/entities/minigames/<name>.rs`, and add `pub mod <name>;` to the minigames module. Define a `pub const ID: &str = "<name>";` at the top — this id is the registry key that `from_id`, `Minigame::id`, `MinigamesResource`, and `setup_minigame_unlocks` all key off of.

2. **Implement the standard interface** — name, description, area, level, `spawn`, `ingest_item`, and the rest of the shape the other minigames implement. Copy the closest existing minigame and adapt; don't deviate from the interface.

3. **Add the variant** to the `Minigame` enum in `src/entities/minigame.rs`.

4. **Update every match statement** in `minigame.rs` that switches over `Minigame` (`id`, the `spawn` dispatch, `ingest_item`, `level`, …). Most are exhaustive, so the compiler lists the non-exhaustive ones — let it drive you through, and don't add a catch-all `_` arm that would silently skip the new variant. **The exception is `from_id`**: it already ends in `_ => None`, so the compiler will *not* force an arm there. Add `<name>::ID => Some(Minigame::<Variant>(...))` by hand — a missing arm means the minigame can never be created by id (no unlock, no levelup respawn), with no warning.

5. **Register it in the unlock graph** — add `unlocks.insert(<name>::ID, ...)` in `setup_minigame_unlocks()`. For an *unlockable* minigame, pass the prerequisites that gate it (which minigames at which levels). For a *startup* minigame (present from the start), pass `Vec::new()` **and** spawn it in `setup_board` (`main.rs`) with `minigames.set_entity(<name>::ID, spawn(...))`, mirroring `button` / `rune` / `primordial_ocean`. No `insert` entry → it never registers; a startup minigame with no `setup_board` spawn → it never appears until something unlocks it.

6. **Register update systems** in `main.rs` if the minigame needs its own per-frame logic (`Update` / `FixedUpdate`). Simple, static minigames may need none.

## Smell tests

- Does `cargo build` succeed with **no** new `match` arms left as `_`? Every switch over `Minigame` should name the new variant explicitly.
- Is the new variant in `setup_minigame_unlocks()`? If not, it can never appear in game.
- Does `from_id` have an explicit `<name>::ID => …` arm? Its trailing `_ => None` means the compiler **won't** flag a missing one — and without it the minigame can't be created by id (no unlock, no levelup respawn).
- Did you define `pub const ID`? And for a *startup* minigame, is it spawned with `set_entity` in `setup_board`? Without that spawn it never appears.
- If it has runtime behavior, are its systems registered in `main.rs`? A module that compiles but isn't registered does nothing.
- Does it implement the **same** interface as its neighbors (name/description/area/level/spawn/ingest_item)? Drift here breaks the common handling in `minigame.rs`.
- `cargo clippy` clean and `cargo fmt` applied (80-col)?
- Recorded a line in today's `logs/YYYY-MM-DD.md` naming the new minigame and why it was added?

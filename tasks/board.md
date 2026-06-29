# Galaxia Task Board

Kanban in markdown. Forward-looking only — this board holds what's relevant now and ahead. Completed work is recorded in `logs/`, not here. When a task lands, delete it from the board and note it in that day's log.

Move tasks down the columns (Backlog → Next → Now) as work progresses. Keep each task small enough that "done" is unambiguous; split anything that isn't.

> Ported from the old `TASKS.md` and validated against the source on 2026-06-20 (`logs/2026-06-20.md`). Every `file.rs:line` below was confirmed still present and accurate at that time.

---

## Now

_(nothing in flight)_

## Next

Ready to pick up — no open prerequisite.

- [ ] **Finish `land`** (tree landed 2026-06-26, life 2026-06-28).
  - `src/entities/minigames/land.rs` — complete the terrain placement logic (stick archaea on a random water-terrain cell); `evolve` runs the sim but has no rendering step, and its `cell_update`/`evolve_fixed_update` aren't registered in `main.rs`.
  - Pattern to follow: `life` is now fully wired — `evolve_fixed_update` (energy+timer gated) and `render_cells` (repaints cells from the model each FixedUpdate) registered in main.rs, `life` in the unlocks table, ingestion seeds cells. Mirror that for land.
  - `land.rs` still starts with `#![allow(warnings)]` — remove and clean the cascade.

## Backlog

Known work, not yet ready or not yet sequenced.

### Minigame features & visuals

- [ ] **Life rule-bender items** — design in `references/life-minigame.md`. Slice 1 (XP = Σ|births−deaths|, geometric levelup) landed 2026-06-28. Remaining:
  - **Extractor item** (slice 2) — a birth on it ejects the item out of the minigame + counts as a birth, and the cell stays empty → a perpetual birth/XP/item engine (the unbounded birth source the level curve needs). Likely becomes the harvest mechanism (replacing click-to-extract).
  - **Other benders** (slice 3) — always-alive, always-dead, teleporter, time-phase (cells updating only on even ticks; needs a tick counter on the minigame). Each is a special `ItemType` a cell holds; `step()` dispatches.
  - **Rune-shaped insertion/extraction** — deterministic placement via runes (the power progression; gives the rune minigame a downstream use).
  - Maybe: reverse rune generation (Life emits a rune if the board forms its shape) — easter-egg; per-tick shape-scanning is expensive, so non-core.
  - Maybe: have a seed *become* the dropped item (apple → apple cell) instead of always Archaea — needs a renders-safely guard since most item types `panic!` in `draw`.
- [ ] **Foundry UI** (`src/entities/minigames/foundry.rs:81-83`) — background graphics, heat-meter visualization, transmutation-timer display.
- [ ] **Missing background visuals** — battery (`battery.rs:70`) and chest (`chest.rs:70`): draw background chest, barrels, etc.
- [ ] **Rune feedback** (`rune.rs:350`) — visual change when the drawing is a valid rune.
- [ ] **Verify "no tint" color handling** — the same TODO sits at `life.rs:220` and `land.rs:366`.
- [ ] **Chest goo check** (`chest.rs:131`) — re-add the commented-out goo material check.

### Systems & performance

- [ ] **Mouse run-conditions** (`src/libs/mouse.rs:184,196`) — replace the TODO links with proper Bevy run conditions (https://bevy-cheatbook.github.io/programming/run-conditions.html).
- [ ] **Area centering** (`src/libs/area.rs:83`) — "center before position" TODO.
- [ ] **ball_breaker** (`ball_breaker.rs:107,130`) — empty out balls as loose items; verify collision works now that the parent is the minigame instead of an aura.

### Item system

- [ ] **Item component updates** (`src/entities/item.rs:28`) — function to alter item components when amount changes.
- [ ] **Rune seeding** (`item.rs:472`) — add runes until there are at least 100.
- [ ] **Mana combining rules** (`item.rs:1000`) — mana-combining rules that can change the mana type.

### Enhancement / polish

- [ ] **Save/load** — not started (`serde` is in `Cargo.toml` but unused for game state). Persist game state and add save/load UI controls.
- [ ] **Documentation** — document the minigame creation process (started in `skills/add-minigame.md`), add ECS architecture diagrams, and inline-document the complex game logic.

### Tech debt / architecture

- [ ] **Item model follow-ups** (the core restructure landed 2026-06-26, see
  `references/item-model.md`): species is still a closed enum — promote to a data registry
  when species count grows; add the derived facet `Flags` bitset when cross-branch
  predicates (edible/flammable/…) are needed; wire `pack()`/`unpack()` into actual save/load.

_(The big Bevy 0.14 → 0.18.1 stack upgrade landed 2026-06-23 — all four steps green. See
`logs/2026-06-23.md`. Verified in the running game 2026-06-24: window renders, B0004 gone,
inventory works.)_

### Bugs

_(none open)_

## Decisions pending

_(none open)_

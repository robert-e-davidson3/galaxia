# Galaxia Task Board

Kanban in markdown. Forward-looking only — this board holds what's relevant now and ahead. Completed work is recorded in `logs/`, not here. When a task lands, delete it from the board and note it in that day's log.

Move tasks down the columns (Backlog → Next → Now) as work progresses. Keep each task small enough that "done" is unambiguous; split anything that isn't.

> Ported from the old `TASKS.md` and validated against the source on 2026-06-20 (`logs/2026-06-20.md`). Every `file.rs:line` below was confirmed still present and accurate at that time.

---

## Now

_(nothing in flight)_

## Next

Ready to pick up — no open prerequisite.

- [ ] **Finish the incomplete minigame implementations.**
  - `src/entities/minigames/life.rs` — implement the missing item-ingestion TODO (fills a random cell); `evolve_fixed_update` is a `return;` stub; `ingest_fixed_update` is empty.
  - `src/entities/minigames/tree.rs:7` — fix `POSITION` (currently `Vec2::ZERO`).
  - `src/entities/minigames/land.rs` — complete the terrain placement logic (stick archaea on a random water-terrain cell); `evolve` runs the sim but has no rendering step.
  - **Wiring gap:** `life`/`land` `cell_update` + `evolve_fixed_update` are NOT registered in `main.rs`, and `life` is not in the `unlocks` table (minigame.rs) so it never spawns. These minigames are dormant until wired up. Their cells render via `Sprite` (the old `Shape` query was corrected 2026-06-24); repaint after evolve via `CellBundle::turn_on/turn_off` + `Query<&mut Sprite, With<Cell>>`.
  - Both files start with `#![allow(warnings)]` — remove and clean the cascade as part of finishing them.

## Backlog

Known work, not yet ready or not yet sequenced.

### Minigame features & visuals

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

- [ ] **Restructure the item model to packed identity + taxonomy** — design agreed in
  `references/item-model.md`. Replace flat `PhysicalForm`/`PhysicalMaterial` enums with a
  `u64` tagged/nested id (closed axes as fields, open species axis as a data registry),
  prefix-mask property helpers (`is_solid`/`state`/`is_fruit`), and a derived facet `Flags`
  bitset. Big change + needs a save migration later; not urgent. Sub-step when it lands:
  replace the chest `can_accept` fruit early-return (added 2026-06-26) with `is_solid()`.

_(The big Bevy 0.14 → 0.18.1 stack upgrade landed 2026-06-23 — all four steps green. See
`logs/2026-06-23.md`. Verified in the running game 2026-06-24: window renders, B0004 gone,
inventory works.)_

### Bugs

_(none open)_

## Decisions pending

_(none open)_

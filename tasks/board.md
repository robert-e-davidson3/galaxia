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
  - `src/entities/minigames/life.rs:324,327` — implement the missing item-ingestion TODO (fills a random cell).
  - `src/entities/minigames/tree.rs:7` — fix `POSITION` (currently `Vec2::ZERO`).
  - `src/entities/minigames/land.rs:227,282,290` — complete the terrain placement logic (290: stick archaea on a random water-terrain cell).

## Backlog

Known work, not yet ready or not yet sequenced.

### Minigame features & visuals

- [ ] **Foundry UI** (`src/entities/minigames/foundry.rs:81-83`) — background graphics, heat-meter visualization, transmutation-timer display.
- [ ] **Missing background visuals** — battery (`battery.rs:70`) and chest (`chest.rs:70`): draw background chest, barrels, etc.
- [ ] **Rune feedback** (`rune.rs:350`) — visual change when the drawing is a valid rune.
- [ ] **Verify "no tint" color handling** — the same TODO sits at `life.rs:220` and `land.rs:366`.
- [ ] **Chest goo check** (`chest.rs:131`) — re-add the commented-out goo material check.

### Systems & performance

- [ ] **Inventory performance** (`src/libs/inventory.rs:306,315`) — short-circuit the item listing instead of iterating past the page, and pre-allocate the result `Vec`.
- [ ] **Inventory scroll bar** — finish scrolling inventories. Partially scaffolded in `inventory.rs`: `ScrollButton` / `ScrollButtonBundle` (left/right arrow sprites) exist and the listing is paginated (`offset` / `per_page`), but no system handles button clicks to change page and the buttons aren't spawned.
- [ ] **Mouse run-conditions** (`src/libs/mouse.rs:184,196`) — replace the TODO links with proper Bevy run conditions (https://bevy-cheatbook.github.io/programming/run-conditions.html).
- [ ] **Area nearest-point** (`src/libs/area.rs:217`) — `nearest` returns cardinal positions only; make it actually nearest.
- [ ] **Area centering** (`src/libs/area.rs:83`) — "center before position" TODO.
- [ ] **ball_breaker** (`ball_breaker.rs:107,130`) — empty out balls as loose items; verify collision works now that the parent is the minigame instead of an aura.

### Item system

- [ ] **Item component updates** (`src/entities/item.rs:28`) — function to alter item components when amount changes.
- [ ] **Rune seeding** (`item.rs:472`) — add runes until there are at least 100.
- [ ] **Mana combining rules** (`item.rs:1000`) — mana-combining rules that can change the mana type.

### Enhancement / polish

- [ ] **Save/load** — not started (`serde` is in `Cargo.toml` but unused for game state). Persist game state and add save/load UI controls.
- [ ] **Documentation** — document the minigame creation process (started in `skills/add-minigame.md`), add ECS architecture diagrams, and inline-document the complex game logic.

## Decisions pending

_(none open)_

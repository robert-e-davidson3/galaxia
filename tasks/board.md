# Galaxia Task Board

Kanban in markdown. Forward-looking only — this board holds what's relevant now and ahead. Completed work is recorded in `logs/`, not here. When a task lands, delete it from the board and note it in that day's log.

Move tasks down the columns (Backlog → Next → Now) as work progresses. Keep each task small enough that "done" is unambiguous; split anything that isn't.

> Ported from the old `TASKS.md` on 2026-06-20 (`logs/2026-06-20.md`). The code references below (`file.rs:line`) came from that list and predate recent item/minigame work — **re-validate each against the current source before picking it up**; some may already be done.

---

## Now

_(nothing in flight)_

## Next

Ready to pick up — no open prerequisite.

- [ ] **Finish the incomplete minigame implementations.**
  - `src/entities/minigames/life.rs` — implement the missing TODO functions (was ~324, 327).
  - `src/entities/minigames/tree.rs` — fix the tree position (was `Vec2::ZERO`, ~line 7).
  - `src/entities/minigames/land.rs` — complete the terrain placement logic (was ~227, 282, 290).

## Backlog

Known work, not yet ready or not yet sequenced.

### Minigame features & visuals

- [ ] **Foundry UI** (`src/entities/minigames/foundry.rs`) — background graphics, heat-meter visualization, transmutation-timer display.
- [ ] **Missing background visuals** — battery (`battery.rs`: barrels, etc.) and chest (`chest.rs`: chest graphics).
- [ ] **Rune feedback** (`rune.rs`) — visual change when the drawing is a valid rune.
- [ ] **Life tint handling** (`life.rs`) — verify the "no tint" color handling.
- [ ] **Chest goo check** (`chest.rs`) — re-add the goo material check.

### Systems & performance

- [ ] **Inventory performance** (`src/libs/inventory.rs`) — short-circuit unnecessary operations; pre-allocate to cut runtime allocations.
- [ ] **Mouse run-conditions** (`src/libs/mouse.rs`) — replace the TODO links with proper Bevy run conditions (https://bevy-cheatbook.github.io/programming/run-conditions.html).
- [ ] **Inventory scroll bar** — add scrolling to inventories.
- [ ] **ball_breaker optimization** (`ball_breaker.rs`) — dispose of balls as loose items; verify collision detection across parent-child entity relationships.

### Item system

- [ ] **Item component updates** (`src/entities/item.rs`) — function to alter item components when amount changes.
- [ ] **Rune seeding** (`item.rs`) — ensure at least 100 runes exist.
- [ ] **Mana combining rules** (`item.rs`) — mana-combining rules that can change mana type.

### Enhancement / polish

- [ ] **Save/load** — use the existing `serde` deps to persist game state; add save/load UI controls.
- [ ] **Documentation** — document the minigame creation process (now started in `skills/add-minigame.md`), add ECS architecture diagrams, and inline-document the complex game logic.

## Decisions pending

_(none open)_

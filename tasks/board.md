# Galaxia Task Board

Kanban in markdown. Forward-looking only — this board holds what's relevant now and ahead. Completed work is recorded in `logs/`, not here. When a task lands, delete it from the board and note it in that day's log.

Move tasks down the columns (Backlog → Next → Now) as work progresses. Keep each task small enough that "done" is unambiguous; split anything that isn't.

> Ported from the old `TASKS.md` and validated against the source on 2026-06-20 (`logs/2026-06-20.md`). Every `file.rs:line` below was confirmed still present and accurate at that time.

---

## Now

_(nothing in flight)_

## Next

Ready to pick up — no open prerequisite.

- [ ] **Draw fallback (do this next).** Make `ItemType::draw`/`palette` return a plain colored square instead of `panic!` for unsupported substances/species, so non-renderable items render harmlessly instead of crashing the game. Robert wants this before broader land testing (tossing e.g. an iron block into land currently crashes). See the Bugs entry for the full picture.

_All three dormant minigames are now wired up and playable — tree (2026-06-26), life (2026-06-28), land v1 (2026-06-29). Other candidates: Life slice 2 (extractor), deepening land, polish._

## Backlog

Known work, not yet ready or not yet sequenced.

### Minigame features & visuals

- [ ] **Life rule-bender items** — design in `references/life-minigame.md`. Slice 1 (XP = Σ|births−deaths|, geometric levelup) landed 2026-06-28. Remaining:
  - **Extractor item** (slice 2) — a birth on it ejects the item out of the minigame + counts as a birth, and the cell stays empty → a perpetual birth/XP/item engine (the unbounded birth source the level curve needs). Likely becomes the harvest mechanism (replacing click-to-extract).
  - **Other benders** (slice 3) — always-alive, always-dead, teleporter, time-phase (cells updating only on even ticks; needs a tick counter on the minigame). Each is a special `ItemType` a cell holds; `step()` dispatches.
  - **Rune-shaped insertion/extraction** — deterministic placement via runes (the power progression; gives the rune minigame a downstream use).
  - Maybe: reverse rune generation (Life emits a rune if the board forms its shape) — easter-egg; per-tick shape-scanning is expensive, so non-core.
  - Maybe: have a seed *become* the dropped item (apple → apple cell) instead of always Archaea — needs a renders-safely guard since most item types `panic!` in `draw`.
- [ ] **Deepen `land`** — v1 landed 2026-06-29 (layered cells, archaea evolve, distinct-type leveling capped at L6, top-layer extraction, top-layer rendering). Design in `references/land-minigame.md`. Remaining: the coexisting food-web evolve rules (plants/animals/mana — who needs/eats what), z-stacked layered rendering (all layers, not just the top), the species-pyramid leveling with rising population bars, and actual plant/animal content.
- [ ] **Foundry UI** (`src/entities/minigames/foundry.rs:81-83`) — background graphics, heat-meter visualization, transmutation-timer display.
- [ ] **Missing background visuals** — battery (`battery.rs:70`) and chest (`chest.rs:70`): draw background chest, barrels, etc.
- [ ] **Rune feedback** (`rune.rs:350`) — visual change when the drawing is a valid rune.
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

- [ ] **Non-renderable items crash on draw.** Most substances/species still `panic!` in `ItemType::draw` — only Mud/Dirt/Sandstone/Salt/Fresh water and Archaea/Apple render. Anything else crashes the moment it's drawn: stored in a chest slot, dropped as a loose item, or (newly easy to hit) tossed into `land` (bulk → terrain → repaint → panic; e.g. an iron block from ball_breaker). Systemic, pre-dates land. Fix: make `draw()`/`palette()` fall back to a placeholder (e.g. a flat colored square) instead of panicking, so unsupported items render harmlessly.

## Decisions pending

_(none open)_

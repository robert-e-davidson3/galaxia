# Life minigame design

Conway's Game of Life as a resource minigame. `src/entities/minigames/life.rs`.

## Loop

- **Seed:** dropping an item into the minigame fills a *random* empty cell with a
  life form. (Energy items are the exception — see below.) Insertion is random by
  design; you can't hand-place patterns (until runes, below).
- **Evolve:** Conway's rules run on a tick cooldown (`EVOLVE_TICKS`, ~1s — a
  countdown rather than an elapsed-time check, so it survives respawn on level-up)
  and *only while the minigame has stored energy*, consuming 1 energy per step.
  Energy comes
  from the foundry (Clicks → Energy). So evolution is something you fuel.
- **Harvest:** clicking a live cell ejects its item (a payout) and clears the
  cell. Harvesting is *not* XP (see below). Later this moves to an extractor item.

The life form is currently **Archaea** (`Discrete · Alive · Microbe`), chosen
because it's one of the few species that actually renders. Cells are
`Option<ItemType>`, not `bool` — this is load-bearing: the rule-bender items below
are just *special ItemTypes* a cell can hold, with `step()` dispatching on them.

## XP and leveling — `XP = Σ |births − deaths|`

Each evolution step earns **`|births − deaths|`** XP (per-step absolute net
change, summed over time). Implemented: `LifeMinigame::step()` returns it,
`evolve_fixed_update` accumulates `xp` and triggers level-up.

Levels are **geometric**: `level_by_xp(xp)` ⇒ level N reached at `xp >= 2^(N-1)`
(level 1 from a single death, level 2 at xp 2, level 3 at 4, …). When `xp` crosses
the next threshold, `evolve_fixed_update` inserts `LevelingUp` and the generic
`minigame::levelup` respawns at the larger grid.

Why this criterion (the others were all gameable):
- **Oscillators, still-lifes, and gliders net zero** — a blinker births 2 and
  kills 2 → `|2−2| = 0`; a block changes nothing → 0; a glider just moves → 0. So
  none of the "free forever" patterns pay anything. Only genuine population
  *change* scores. (Ticks / total-deaths / total-births alone were all farmable by
  a single oscillator.)
- **Works at 1×1** — a lone seeded cell dies → `|0−1| = 1` XP. So the 1×1 start is
  kept (no need to start bigger).
- **The difficulty curve is built in,** with one symmetric formula, because the
  *economy* is asymmetric: deaths are a **bounded well** (you can only kill what
  you seeded, and seeding is random + rate-limited), while an engineered **net
  birth surplus is unbounded** — but impossible in vanilla Conway on a bounded
  board. So early levels come from death-farming; once thresholds outrun your
  death rate, the only way forward is a birth engine, which needs the rule-benders.

## Rule-bender items (roadmap)

These break vanilla Conway to make a net-birth surplus possible. Each is a special
`ItemType` a cell holds; `step()` dispatches on it.

- **Extractor (next — "slice 2"):** a birth that would land on an extractor cell is
  instead *ejected from the minigame* (item output) and the cell *stays empty* —
  and it still counts as a birth for XP. Because the cell stays empty, if its
  neighborhood keeps satisfying the birth rule it births *every step*: a perpetual
  stream of items + XP from one tile. This is the unbounded birth source the curve
  demands, and it unifies harvest + XP + the births-surplus into one item. It's a
  pure *sink* (extractor-births don't populate, so they don't propagate) — so it
  drains its neighbors rather than growing the colony; feeding it from a generator
  it doesn't consume is the puzzle.
- **Other benders ("slice 3"):** always-alive, always-dead, teleporter (move life
  across the board), time-phase (cells that update only on even ticks → needs a
  tick counter on the minigame). All "modded Conway" pieces for engineering
  surpluses.
- **Runes shape insertion (and later extraction):** unlock deterministic placement
  — drop a rune-defined shape (blinker → glider → eventually a gun) instead of a
  random cell. This is the *power* progression and gives the rune minigame a
  downstream use. Same mechanism shapes extraction (extractor + rune = harvest a
  region).
- **Reverse (Life → runes by producing a shape):** wacky idea — generate a rune if
  the board ever forms its shape. Probably not core: scanning the board for shapes
  every tick is expensive, and under random insertion players can't aim for one.
  Maybe a rare easter-egg (scan occasionally, big reward).

## Notes / open

- Level-up **preserves the colony** — `levelup()` copies the old cells into the
  larger new grid (top-left aligned).
- Births inherit a live neighbor's species (all Archaea today, so moot until
  varied life renders).
- Seeding places **Archaea regardless of the item dropped** (the item is the
  "nutrient"); Archaea is used because it's render-safe. If we want dropped items
  to *become* the life form (apple → apple cell), seeding needs a renders-safely
  guard (most item types still `panic!` in `draw`).

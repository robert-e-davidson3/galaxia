# Item model (packed identity + taxonomy)

**Status: design, not yet implemented.** This is the agreed target for restructuring
`src/entities/item.rs`. The current code uses flat enums (`PhysicalForm`,
`PhysicalMaterial`, …) matched ad-hoc; this document is what we are migrating
toward. See `tasks/board.md` for the implementation task.

## Why

The current `ItemType` packs several different *kinds* of field into single flat
enums. `PhysicalForm` mixes matter-state (Gas/Liquid/Powder), generic solid shape
(Lump/Block/Ball/Ore), terrain, and open-ended identity (Apple/Lemon/Lime) on one
axis; `PhysicalMaterial` mixes substance (Iron/Mud) with life-stage
(Seed/Adult/Fruit). Consequences we hit:

- `#[repr(u8)]` caps `PhysicalForm` at 256, but specific species (apples, oaks,
  fish, …) are an open set that will far exceed that.
- "Is an apple solid?" has no answer in the type — chest `can_accept` hardcodes a
  form allowlist, so the (solid) apple read as not-solid and wasn't storable.
- Exhaustive `match`es (`draw`/`identifier`/`palette`) `panic!` on unhandled
  variants; adding the first concrete species (Apple) crashed because
  `identifier()` had no arm for it.

The numeric reprs were speculative (no casts/serde today). Identity actually
lives in the string `uid()` and `std::mem::discriminant`.

## Core ideas

1. **Closed taxonomic axes vs. open identity.** Matter-state, mana element,
   energy kind, life-class, life-stage are small, stable, exhaustive-match-
   friendly — keep them as enums/fields. *Which species* is unbounded and
   data-driven — it gets a wide numeric leaf, never an enum-variant-per-species.
2. **Taxonomy (single-parent tree) vs. facets (cross-cutting).** A taxonomy nests
   (`Discrete → Inanimate → Fruit → Apple`) and is encoded as nested prefix
   bit-fields, so `is_fruit()` is a prefix mask. Facets that cross branches
   (`edible`, `flammable`, `metal`) that don't nest live in a separate derived
   `Flags` bitset computed from the id — not baked into identity.
3. **Class (identity) vs. instance data.** The packed id below is the immutable
   *class*: identity for equality, hashing, and compact save/load. Per-item data
   (`amount`, cosmetic variant, world transform/velocity) lives on the `Item`
   instance / ECS components. Instance data **is** persisted to disk on save — it
   just isn't part of the class id and doesn't affect type identity.

## Encoding invariant

A class id is a **`u64`**, read **MSB → LSB**, as a **tagged, nested union**: each
tag field's value determines the meaning — and the existence — of every field
after it. The same bit positions mean different things under different tags
(e.g. Physical bits [55:52] are "substance class" under Bulk but part of
"animacy/class" under Discrete), so any property mask must include the tag fields
that gate it. Masks may be non-contiguous. Unused trailing bits are fallow and
zeroed.

`u64` (not `u32`): the deepest path needs ~38 bits and we want generous headroom.
If we ever need tighter packing it's a migration; the hope is 64 bits never
forces one.

## Layout

### Top tag

| field  | width | bits    | values |
|--------|-------|---------|--------|
| domain | 3     | [63:61] | `000` Physical, `001` Mana, `010` Energy, `011` Abstract, `100` Minigame; `101`/`110` spare; **`111` = escape: does not follow the taxonomy** (remaining 56 bits are a free-form unique id) |

### Physical  (domain `000`, payload [60:0])

| field     | width | bits    | values |
|-----------|-------|---------|--------|
| structure | 5     | [60:56] | Gas, Liquid, Powder, Bulk, Discrete, Terrain (6 used, room to 32) |

**If structure is Bulk-like** (Gas / Liquid / Powder / Bulk / Terrain) — identity
is the *substance*:

| field           | width | bits    | notes |
|-----------------|-------|---------|-------|
| substance class | 4     | [55:52] | Earthen, Metal, Gem, Organic, Water, Exotic (maskable; derivable from substance — kept for masking) |
| substance       | 8     | [51:44] | the specific material (Iron, Mud, …) |
| processing      | 3     | [43:41] | Ore(raw), Refined, Worked, … — refinement state (Bulk only) |
| shape           | 3     | [40:38] | Lump, Block, Ball, Gravel, … — geometry (Bulk only) |
| quality/grade   | 4     | [37:34] | purity / clarity / grade |
| fallow          | 34    | [33:0]  | |

(An iron *ore* = substance=Iron, processing=Ore, shape=Gravel. A refined iron
block = substance=Iron, processing=Refined, shape=Block.)

**If structure is Discrete** — identity is the *species*:

| field   | width | bits    | values |
|---------|-------|---------|--------|
| animacy | 3     | [55:53] | Alive, Inanimate (room for more, e.g. Construct/Undead) |
| class   | 4     | [52:49] | Alive: Microbe, Plant, Animal · Inanimate: Fruit, Tool, Weapon, … |
| state   | 7     | [48:42] | meaning depends on class — Alive: life-stage (Seed/Baby/Youth/Adult/Elder/Corpse); Fruit: **freshness** 0 (spoiled) … 127 (fresh); stateless classes leave it 0 |
| species | 20    | [41:22] | OPEN leaf (~1M) — the unbounded axis |
| fallow  | 22    | [21:0]  | |

Note: because `state` is in the id, a state change (a fruit's freshness dropping,
an organism aging) changes the item's class id — apples at different freshness are
different types and don't stack. Intentional: a storage that holds a single type
will **average** freshness across the stack (Wurm-style) to collapse them back to
one value, so full freshness stays tenable when storage is full. Until that
averaging exists, freshness is used as effectively binary (all-0s spoiled /
all-1s fresh).

### Mana  (domain `001`, payload [60:0])

| field   | width | bits    | values |
|---------|-------|---------|--------|
| element | 3     | [60:58] | Fire, Water, Earth, Air, Light, Dark |
| intent  | 2     | [57:56] | Attack, Defense, Support |
| subkind | 8     | [55:48] | |
| fallow  | 48    | [47:0]  | |

(element+intent = the 5 "broad type" bits.)

### Energy  (domain `010`, payload [60:0])

Energy is a **bitmask of the categories an item works for** (items may combine):

| field      | width | bits    | values |
|------------|-------|---------|--------|
| kinds mask | 6     | [60:55] | Kinetic, Potential, Thermal, Electric, Magnetic, Radiant (OR'd) |
| fallow     | 55    | [54:0]  | |

### Abstract  (domain `011`, payload [60:0])

| field | width | bits    | notes |
|-------|-------|---------|-------|
| kind  | 13    | [60:48] | broad abstract kind — many expected (Click, XP, Rune, currencies, achievements, stats, …). domain+kind = 16 bits. |

Per-kind payload in [47:0] (48 bits):

| kind  | field   | width | bits    |
|-------|---------|-------|---------|
| Click | variant | 2     | [47:46] |
| XP    | tier    | 4     | [47:44] |
| Rune  | rune    | 7     | [47:41] | (≥100 runes planned) |

### Minigame  (domain `100`, payload [60:0])

| field    | width | bits    | notes |
|----------|-------|---------|-------|
| which    | 13    | [60:48] | >100 minigames planned, ample room |
| subclass | 48    | [47:0]  | reserved for later sub-classification |

## Instance data (persisted, not part of the class)

Saved with each item but outside the class id, so it doesn't affect identity,
equality, or how items stack by type:

- `amount: f32`
- cosmetic variant (color/pattern within a substance or species)
- world state (transform, velocity, …) — ECS components

## Worked examples

| item             | class id |
|------------------|----------|
| fresh apple      | Physical · Discrete · Inanimate · Fruit · freshness=127 · species=Apple |
| spoiled apple    | Physical · Discrete · Inanimate · Fruit · freshness=0 · species=Apple |
| apple tree       | Physical · Discrete · Alive · Plant · state=Adult · species=AppleTree |
| iron ore         | Physical · Bulk · class=Metal · substance=Iron · processing=Ore · shape=Gravel |
| iron block       | Physical · Bulk · class=Metal · substance=Iron · processing=Refined · shape=Block |
| fire-attack mana | Mana · element=Fire · intent=Attack · subkind=0 |
| rune #12         | Abstract · kind=Rune · rune=12 |

## Property queries

Taxonomic predicates are prefix masks over the tag fields (include every gating
tag, since bit positions are reused across tags):

- `is_alive` = (domain=Physical, structure=Discrete, animacy=Alive)
- `is_metal` = (domain=Physical, structure∈Bulk-like, substance class=Metal)
- `is_fruit` = (domain=Physical, structure=Discrete, animacy=Inanimate, class=Fruit)
- freshness is a *value* field, not a tag — `is_spoiled` = `is_fruit` AND
  freshness==0 (a field comparison, not a pure prefix mask).

Cross-cutting facets (`edible`, `flammable`, `combustible`, `magnetic`, …) don't
nest, so they are **not** in the id. Derive a `Flags(u64)` bitset from the id via
`match`/table and AND against it: `flags(id) & (EDIBLE | PERISHABLE)`.

## Decisions

- **Fruit is a category, not a life stage.** It lives under `Discrete · Inanimate`
  (a harvested product), with `Apple` as the species leaf and the `state` field
  carrying fruit condition (Fresh/Rotting). `is_fruit()` is a prefix mask.
- **`state` is one field whose meaning is class-dependent** — life-stage under
  Alive; a 7-bit **freshness** scale (0 spoiled … 127 fresh) under Fruit; unused
  otherwise. Putting it in the id means state changes rewrite the class id
  (accepted); single-type storage averages freshness Wurm-style so stacks don't
  fragment, and freshness is used as binary (all-0s/all-1s) until that lands.
- **Shape and processing are separate Bulk axes.** processing/refinement
  (Ore=raw → Refined → Worked) is distinct from geometric shape
  (Lump/Block/Ball/Gravel). An ore is still of a specific substance.
- **Quality/grade** is a Bulk axis (purity/clarity/grade).
- **Cosmetic variant is instance data**, not part of the class id.
- **Substance class and life class are derivable** from substance/species
  (Iron⇒Metal). Kept as prefix fields only so the `is_*` checks are masks; the
  redundancy must stay consistent (never Iron+Gem).
- **Terrain is a `structure`** (bulk-like) for now.
- **Validity is separate from the taxonomy.** The id space allows nonsensical
  combos (gaseous granite); we just never construct them (or add `is_valid()`).
- **Domain `111` is an escape hatch** for items that don't follow the taxonomy;
  the other 56 bits are then a free-form unique id.

## Stability & migration

Because class/structure/state are prefix bits, a species' id *includes* its
classification — so reclassifying a species (or inserting an intermediate
category) changes its id and breaks old save files. We accept that model changes
require a save migration; we'll minimize them and expect the first ones only
after release. Species numeric ids should otherwise be treated as stable (don't
renumber casually).

## Implementation notes (when we build it)

- Replace the string `uid()`/`discriminant` identity with the packed `u64`.
- Keep exhaustive `match`es only on the *closed* axes (structure, classes,
  stages); the open species axis becomes data (a registry keyed by stable id),
  not enum variants — so adding the 257th species is data, not a recompile and a
  new match arm.
- Replace ad-hoc form allowlists (e.g. chest `can_accept`, the current fruit
  early-return) with prefix-mask helpers (`is_solid()` / `state()` / `is_fruit()`).

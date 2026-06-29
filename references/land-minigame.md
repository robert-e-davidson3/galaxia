# Land minigame design

A terraform-and-evolve ecosystem. `src/entities/minigames/land.rs`. Description:
"Evolve life." Unlocks from the Primordial Ocean. More involved than `life`
(which is abstract Conway); Land is a layered ecosystem with terrain that life
depends on.

## Layered cells

A cell is a **stack of coexisting layers**, one occupant per layer:

```
struct LandCell {
    terrain: ItemType,        // always present; default Mud
    micro:   Option<ItemType>,
    plant:   Option<ItemType>,
    animal:  Option<ItemType>,
    other:   Option<ItemType>,
}
```

The layers **are the item-model taxonomy classes**, so insertion routes by the
item's `class()` automatically:
- **terrain** ← Physical `Bulk` (a substance: Mud, water, stone, …). Default Mud.
- **micro** ← Discrete, class `Microbe` (Archaea, Bacterium)
- **plant** ← class `Plant` (Algae, Grass, Fern, Bush, Tree)
- **animal** ← class `Animal` (Insect, Fish, Amphibian, Reptile, Mammal, Bird)
- **other** ← everything else (Mana/magical — spells, monoliths — *and* a
  catch-all you can stash any item in, e.g. toss an apple in there)

Populations are counted across cells (one occupant per layer per cell), so "many
plants" means many tiles with a plant.

## Loop

- **Terraform:** insert a liquid (→ water) or solid (→ mud/stone) — it replaces a
  cell's terrain layer.
- **Seed life:** there is **no spontaneous generation** — you bring organisms
  from elsewhere (Archaea harvested from the `life` minigame) and insert them;
  they route to their class's layer. (Life → Land pipeline.)
- **Stash:** any non-terrain, non-organism item goes in the `other` layer.
- **Evolve:** fueled by stored energy (consumes per step, on a tick cooldown like
  `life`).
- **Extract:** clicking a tile removes the occupant of the **highest occupied
  layer first** (other → animal → plant → micro); terrain is the base and stays.

## Evolve — a food web (full vision)

Each layer's rules read the layers below/around it (this is the big creative +
code chunk):
- microbes need/condition the terrain (archaea need water or die; spread to empty
  water neighbors);
- plants need water terrain (+ maybe microbes to enrich soil);
- animals eat plants or other animals;
- `other` (magic) bends the rules.
Higher species **coexist** with lower ones (a real pyramid/food web), they don't
replace them.

## Leveling (full vision)

By **species established**, with a population bar that **rises with complexity** —
1 archaea, ~several bacteria, ~many algae/plants, … So the level reflects a deep,
diverse, thriving ecosystem (the pyramid), and the rising bars stretch the ~13
species into a long progression. Grid **grows like the other minigames** — the
shared `_blocks_per_row`/`_blocks_per_column` (1×1, 2×1, 2×2, 3×2, 3×3, …) — so
habitat and ecosystem advance together.

## Rendering (full vision)

Z-stacked layers per cell: terrain at the bottom z, then micro → plant → animal →
other on top.

## "Other" hooks Land into mana/runes

The `other` layer is where monoliths (rune-placed structures) and spells (mana
effects on a region) live — giving those systems a downstream use, like
rune-shaped insertion does for `life`.

---

## v1 scope (implemented — "ready now, capped low")

A minimal but functional Land, deliberately capped at a low level (content is
thin — only mud/water terrain and archaea exist as life). Everything below is the
v1; the rest above is the future.

- **Cell model:** the full 5-layer `LandCell` (so it's extensible), but only
  `terrain`, `micro`, and `other` are populated in practice (no plant/animal
  content yet).
- **Grid:** switch off the broken `4 * level` (which is 0×0 at level 0) to the
  shared `_blocks_per_row`/`_blocks_per_column` formula (1×1, 2×1, 2×2, …). One
  sprite per cell (the spawn loop must size off the *model grid*, not the pixel
  area — the old code's mismatch is why it never worked).
- **Insertion / routing:** energy → energy pool; Bulk liquid → water terrain,
  Bulk solid → its substance as terrain; Discrete organism → its class layer
  (only Microbe/archaea relevant now); anything else → `other`. Places one unit on
  a random cell, ejects the remainder (as today).
- **Evolve:** archaea rule only — an archaea on non-water terrain dies; otherwise
  it spreads to a random neighbor that is water + empty of micro. Energy-fueled,
  tick cooldown (mirror `life`'s `evolve_fixed_update`).
- **Leveling (v1, simple):** level = number of distinct item types present across
  all layers, capped at a low `MAX_LEVEL`. So diversifying (mud → +water →
  +archaea → +stashed item) grows the grid to a playable size, then caps. (The
  full species-pyramid leveling is for later.)
- **Extraction:** click removes the topmost occupied non-terrain layer (other →
  animal → plant → micro) and ejects it; terrain stays.
- **Rendering (v1):** one sprite per cell showing the **topmost occupied layer**
  (or the terrain if empty). Full z-stacked layers are a later polish.
- **Wire-up:** register `cell_update` + `evolve_fixed_update` (+ a render system)
  in `main.rs`; `land` is already in the unlocks table; remove
  `#![allow(warnings)]`.

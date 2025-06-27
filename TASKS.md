# Galaxia Task List

## High Priority Tasks (Critical)

### 1. Complete incomplete minigame implementations
- **File**: `src/entities/minigames/life.rs:324,327` - Implement missing TODO functions
- **File**: `src/entities/minigames/tree.rs:7` - Fix tree position (currently Vec2::ZERO)
- **File**: `src/entities/minigames/land.rs:227,282,290` - Complete terrain placement logic

## Medium Priority Tasks (Important)

### 1. Implement missing foundry UI features
- **File**: `src/entities/minigames/foundry.rs:81-83`
- Add background graphics
- Implement heat meter visualization
- Add transmutation timer display

### 2. Improve inventory system performance
- **File**: `src/libs/inventory.rs:306,315`
- Rewrite inventory logic to short-circuit unnecessary operations
- Pre-allocate data structures to reduce runtime allocations

### 3. Add missing visual elements for minigames
- **File**: `src/entities/minigames/battery.rs:70` - Add background elements (barrels, etc.)
- **File**: `src/entities/minigames/chest.rs:70` - Add background chest graphics

### 4. Implement mouse system improvements
- **File**: `src/libs/mouse.rs:184,196`
- Replace current TODO links with proper Bevy run conditions
- Reference: https://bevy-cheatbook.github.io/programming/run-conditions.html

### 5. Add scroll bar to inventories

## Low Priority Tasks (Enhancement)

### 1. Optimize ball_breaker minigame
- **File**: `src/entities/minigames/ball_breaker.rs:107,130`
- Implement ball disposal as loose items
- Verify collision detection works with parent-child entity relationships

### 2. Add comprehensive documentation
- Document minigame creation process and patterns
- Create architecture diagrams for the ECS system
- Add inline documentation for complex game logic

### 3. Implement save/load functionality
- Utilize existing serde dependencies for serialization
- Create game state persistence system
- Add save/load UI controls

## Additional TODOs Found in Code

### Item System
- **File**: `src/entities/item.rs:28` - Add function for altering item components when amount changes
- **File**: `src/entities/item.rs:472` - Add runes until there are at least 100
- **File**: `src/entities/item.rs:1000` -Add weird mana combining rules that can change mana type

### Minigame-Specific
- **File**: `src/entities/minigames/rune.rs:350` - Add visual change when drawing is a valid rune
- **File**: `src/entities/minigames/life.rs:220,366` - Verify "no tint" color handling
- **File**: `src/entities/minigames/chest.rs:131` - Re-add goo material check

---

**Total Lines of Code**: ~7,200 lines across 42 Rust files
**Last Updated**: 2025-06-26

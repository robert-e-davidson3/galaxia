# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working in this repository.

Galaxia is a Rust game built on the Bevy engine: a collection of interconnected minigames bound into one overarching world. It's a vehicle for learning Rust and game design, so favor clear, idiomatic code over clever code.

## Directory layout

- `logs/` — One file per day (`YYYY-MM-DD.md`), filled out as work happens. Each entry describes what changed, what was researched, and the _why_ behind decisions. Append-only history; search these when you need context on how something came to be.
- `references/` — Definitive, current facts about the repo, engine, and conventions. A single narrative per topic, kept up to date. Start with `references/repo-layout.md` (the source map); see also `local-dev.md` (build/run/test), `tech-stack.md` (dependencies), and `code-style.md`. When a fact in a log is now settled, promote it here.
- `skills/` — Procedures to follow under specific conditions, written so they can be run cold. Start with `skills/add-minigame.md`.
- `tasks/board.md` — Forward-looking kanban (Now / Next / Backlog). Completed work is recorded in `logs/`, not here — when a task lands, delete it from the board and note it in that day's log.

## How to work here

- Before non-trivial work, check `references/` for settled facts and `logs/` for recent context. Skim `tasks/board.md` for what's in flight.
- When you make a decision or learn something non-obvious, write it to today's log. If it's a durable fact (not over-time nuance), also update or create the relevant `references/` file.
- When you find yourself running the same procedure twice, write it up as a skill.
- Adding a minigame? Follow `skills/add-minigame.md`.

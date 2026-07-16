# GMM rebuild — continuation brief

Project: `C:\Users\kp\Desktop\workspace\gmm` — Tauri desktop mod manager for Zenless Zone Zero (ZZZ).

## Decision: full rebuild from scratch (not a migration)

Scanned the existing repo and it's worse than a normal migration target:
- `src-tauri/src/{db,scanner,models,error,launcher,constants}.rs` are dead/orphaned — never `mod`-declared, don't compile into the binary. The real app is one 4,887-line `main.rs`.
- The React→Vue migration is ~80% done but currently broken — `router/index.js` imports `pages/*.vue` files that don't exist (only `.jsx` remain).

Given how much needed touching anyway, starting fresh is cheaper than untangling this.

## New stack
Tauri v2 + Vue 3 + TypeScript. **Tailwind dropped from scope** (too token-heavy for the gain, user's call).

## Feature scope (all confirmed with user)

1. **ZZZ only** — drop Genshin/HSR/WuWa entirely (multi-game switching, per-game DB files, Genshin-specific Traveler migration logic, 3 of 4 definition TOMLs).
2. **Vue 3 + TypeScript** — fresh, no legacy JS/JSX debt.
3. **Tauri v2** — fresh setup, not a v1→v2 port.
4. **Self-update** — old app has updater config (pubkey + endpoint) in `tauri.conf.json` but it's never wired to the frontend. Build it properly this time.
5. **User-addable agents (characters)** — let the user add new ZZZ agents themselves via UI. **Confirmed: ship the built-in ZZZ roster as seed data, plus let users add more on top** (not fully-empty/user-only).
6. **Alias-array slug recognition** — replace the old 13-priority fuzzy name-matching heuristic (`find_entity_slug_from_hint`) with an explicit `aliases: string[]` per agent (e.g. Ellen Joe → `["ellen","ellen joe","ellenjoe","joe"]`). Scanner matches mod-folder/ini hints against this list directly. Needed because heuristics tuned for official names won't generalize to user-added agents.
7. **Mod grouping** — merge/group multiple mod folders into one logical unit in the UI. **Confirmed: virtual grouping only** — folders stay separate on disk, DB gets a group/parent relation, UI treats them as one toggle unit (not a physical file merge).
8. **In-game skin-toggle memory** — user wants GMM to remember per-mod in-game toggle state (e.g. hat on/off, shirt variant) currently controlled via 3DMigoto hotkeys inside the game. NOT the same as "remember enabled/disabled" (that already persists via the `DISABLED_` filename prefix). The old code already parses `; Constants`/`[Key...]` sections from mod `.ini` files for a "keybinds" popup — that's the hook point. Would need to read the ini's `Constants` default value, expose it as a toggle in GMM, rewrite that default into the `.ini` before each game launch, and persist the choice in the DB. **Flagged as needing a research spike against real mod files before sizing** — highest-risk item on the list.

## Phased build plan (rough token estimates, each phase ~its own session)

1. Scaffold: Tauri v2 + Vue 3 + TS project, DB schema from scratch, Pinia, router, base layout (reuse old CSS as visual reference) — 80–150K
2. Agents + aliases: `agents` + `agent_aliases` tables, seed script for built-in ZZZ roster, Add/Edit Agent UI — 100–180K
3. Scanner rewrite: port ini-parsing/archive-import logic as reference, rewrite matching against alias arrays (simpler than old heuristic), keep a category-fallback bucket — 100–150K
4. Mod management UI: cards, enable/disable, presets, import modal, keybinds popup, launcher — 150–220K
5. Mod grouping (virtual): group table/relation, merge/split UI, group-level toggle — 80–150K
6. Self-update: Tauri v2 updater plugin + release endpoint + UI — 40–70K
7. Skin-toggle memory: research spike first (check real mod ini conventions), then persisted state + ini-patch-before-launch + toggle UI — 100–300K+ (unknown until spike)

**Total ballpark: ~650K–1.2M tokens** for the whole thing done carefully.

## Where to start when resuming
Phase 1 (scaffold) + Phase 2 (agent CRUD + aliases) together — everything else depends on that data model. Confirm the user still wants this exact scope before writing any code.

# GMM Rebuild — Phase 1: Scaffold

## Context

`C:\Users\kp\Desktop\workspace\GMM` is a Tauri v1 + React mod manager for gacha games. Per `REBUILD_NOTES.md`, the decision (already confirmed with the user in a prior session) is a **full rebuild, not a migration**: the old `src-tauri/src/main.rs` (4,886 lines, dead orphaned modules never `mod`-declared) and the half-finished React→Vue port are being replaced outright with **Tauri v2 + Vue 3 + TypeScript, ZZZ-only, no Tailwind**.

This is Phase 1 of 7: scaffold the new project skeleton — fresh Tauri v2 + Vue3 + TS app, a from-scratch DB schema, Pinia, router, and a base layout — so Phases 2–7 (agents/aliases, scanner, mod management UI, grouping, self-update, skin-toggle memory) have a skeleton to build features into. No feature logic (scanning, importing, agent CRUD) is implemented in this phase — that's later phases, per the notes' own token-budget breakdown.

We're on branch `upgrade/tauri-v2-vue-refactor`. Toolchain confirmed present: Node v22.18.0, npm 10.9.3, cargo/rustc 1.92.0 (well above Tauri v2's 1.77.2 minimum).

## What gets removed, kept, or archived

**Removed** (old React frontend, fully superseded) — already done:
- `src/` entirely (`App.jsx`, `main.jsx`, `App.css`, `pages/*.jsx`, `components/*.jsx`, `contexts/SettingsContext.jsx`, `utils/localStorage.js`) — React is gone, Vue replaces it 1:1 conceptually (pages → routes, contexts → Pinia stores, components rebuilt later per-phase).
- `src-tauri/definitions/genshin.toml`, `hsr.toml`, `wuwa.toml` — multi-game dropped per REBUILD_NOTES point 1. `zzz.toml` is **kept** (seed data for Phase 2, untouched this phase).
- Old `package.json` React deps (react, react-dom, react-router-dom, react-select, react-toastify, react-window, recharts, framer-motion, lucide-react, @vitejs/plugin-react) — still to be removed when package.json is rewritten.

**Archived, not deleted** (needed as reference in later phases) — already done:
- `src-tauri/src/main.rs` → moved to `src-tauri/_legacy/main.rs.reference` (outside `src/` so Cargo won't try to compile it). Phase 3 needs its scanner/archive-import logic as reference; Phase 7 needs its `; Constants`/`[Key.*]` ini-parsing (lines ~4283-4400) for skin-toggle memory. Kept read-only, never wired into the new build.

**Kept as-is:**
- `vault/` (mods storage dir, already gitignored).
- `src-tauri/icons/` (bundle icons — same app, same icon).
- `public/fontawesome/` (locally bundled Font Awesome — old layout's icon-font `<i>` tags are ported into the new Sidebar/layout, so this stays linked from `index.html`).
- `public/images/` (entities/filters/logos/placeholder art — game asset data for later phases).
- Git history, branch, remotes.

## New DB schema (`src-tauri/src/db/schema.rs`)

Old schema (`categories → entities → assets`, `settings`, `presets`/`preset_assets`) was multi-game generic. New schema is ZZZ-only and adds the two new concepts from REBUILD_NOTES (agent aliases, mod grouping) all in one pass, so later phases build CRUD against an already-stable schema instead of migrating it repeatedly:

```sql
PRAGMA foreign_keys = ON;

CREATE TABLE categories (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    slug TEXT UNIQUE NOT NULL
);

CREATE TABLE agents (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    slug        TEXT UNIQUE NOT NULL,
    description TEXT,
    details     TEXT,
    base_image  TEXT,
    is_builtin  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE agent_aliases (
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id INTEGER NOT NULL,
    alias    TEXT NOT NULL,
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE,
    UNIQUE (agent_id, alias)
);

CREATE TABLE mods (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id       INTEGER,   -- nullable: UI/uncategorized mods
    category_id    INTEGER,
    name           TEXT NOT NULL,
    description    TEXT,
    folder_name    TEXT NOT NULL UNIQUE,
    image_filename TEXT,
    author         TEXT,
    FOREIGN KEY (agent_id)    REFERENCES agents(id)     ON DELETE SET NULL,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL
);

CREATE TABLE mod_groups (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL
);

CREATE TABLE mod_group_members (
    group_id INTEGER NOT NULL,
    mod_id   INTEGER NOT NULL,
    PRIMARY KEY (group_id, mod_id),
    FOREIGN KEY (group_id) REFERENCES mod_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (mod_id)   REFERENCES mods(id)        ON DELETE CASCADE
);

CREATE TABLE presets (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT UNIQUE NOT NULL,
    is_favorite INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE preset_mods (
    preset_id  INTEGER NOT NULL,
    mod_id     INTEGER NOT NULL,
    is_enabled INTEGER NOT NULL,
    PRIMARY KEY (preset_id, mod_id),
    FOREIGN KEY (preset_id) REFERENCES presets(id) ON DELETE CASCADE,
    FOREIGN KEY (mod_id)    REFERENCES mods(id)     ON DELETE CASCADE
);

CREATE TABLE settings (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
```

Not created this phase: a skin-toggle-memory table (REBUILD_NOTES point 8 explicitly flags that as needing a research spike before sizing — schema TBD in Phase 7, not guessed now). `mod_group_members` uses composite PK, same pattern as old `preset_assets`.

## New frontend structure

```
src/
  main.ts                       — Vue app bootstrap, Pinia + router install
  App.vue                       — root: renders AppShell
  layouts/AppShell.vue          — sidebar + main-content flex layout (ported CSS shape)
  components/layout/Sidebar.vue — logo, nav items, placeholder preset list
  pages/                        — file-based routing (consistent with your other Vue projects)
    index.vue                   — Dashboard placeholder
    agents/index.vue            — Agents list placeholder
    agents/[slug].vue           — Agent detail placeholder
    presets.vue                 — Presets placeholder
    settings.vue                — Settings placeholder
  stores/
    agents.ts, mods.ts, presets.ts, settings.ts  — empty Pinia stores (state shape only, no backend calls yet)
  styles/main.css                — global CSS custom properties + base layout rules, ported from old main.css
  types/index.ts                  — Agent/Mod/Preset/ModGroup TS interfaces mirroring the new schema
```

**Routing**: vue-router v5's built-in file-based routing (merged from the formerly-separate `unplugin-vue-router` — `vue-router/vite` plugin + `vue-router/auto-routes`), same convention as this user's `create-vue-boilerplate` skill — kept for consistency across Vue projects even though this one has no LIFF/Tailwind/business-specific pieces. `VueRouter()` must come before `vue()` in the Vite plugins array (hard requirement, not style). No auto-import/auto-component codegen plugins — kept out of scope for a small solo-desktop-app scaffold (explicit imports instead).

**State**: Pinia only (no TanStack Query — this app talks to the Tauri backend via IPC `invoke`, not HTTP, so there's no server-cache-style fetching layer to justify it).

**Package manager**: npm (existing `package-lock.json` convention retained; old lockfile discarded/regenerated since deps change completely).

### Base layout — visual reference carried over from old CSS

Old `main.css` is a fixed dark theme (not `prefers-color-scheme` toggled) with this palette, carried into the new `styles/main.css` as CSS custom properties:

```
--primary: #9c88ff   --secondary: #8c7ae6
--dark: #1e1e2e      --darker: #181825      --light: #f5f6fa
--accent: #ff9f43    --danger: #ff6b6b      --success: #1dd1a1
--card-bg: rgba(30,30,46,0.7)
--zzz-electric: #a176e7  --zzz-fire: #ff7a3d  --zzz-ice: #99d8f0
--zzz-ether: #d17df5     --zzz-physical: #c2c8ce
--sidebar-width: 260px
```

Layout shape: `.app-container` flex row → `.sidebar` (260px, `--dark` bg, logo/nav/preset-list) + `.main-content` (flex:1, 25px padding, own scroll). Font stack `"Segoe UI", Tahoma, Geneva, Verdana, sans-serif`. Card style: translucent `--card-bg`, 12px radius, hairline `rgba(255,255,255,0.1)` dividers, thin custom-styled scrollbar. This phase only builds the shell (Sidebar + empty page containers styled to this system) — actual ModCard/EntityCard/etc. components are Phase 3/4 work.

Not carried over: WuWa/elemental (Genshin) color vars (multi-game dropped), the unused React/Vite boilerplate `App.css` (dead file, was never the real app's styling).

## New Tauri v2 backend structure

```
src-tauri/
  Cargo.toml         — rewritten for Tauri v2 (tauri 2.x, tauri-build 2.x, tauri-plugin-dialog,
                        tauri-plugin-fs, tauri-plugin-shell, tauri-plugin-updater — v2 split the old
                        monolithic allowlist into per-feature plugins). Archive/ini/regex deps
                        (zip, sevenz-rust, unrar, rust-ini, toml, walkdir) kept in Cargo.toml
                        even though unused until Phase 3, so the dependency graph doesn't shift later.
  tauri.conf.json    — converted to v2 config schema. Fixes two stale fields found in the old
                        config: bundle identifier `GenshinModManager` → something like
                        `com.khawpunkp.gmm`, and window title `"ZMM - A Tauri Mod Manager"` → `"GMM"`.
                        Window size/min-size (1400x800, min 900x650) and updater pubkey/endpoint
                        carried over unchanged (same signing key, same update feed).
  capabilities/default.json — v2 permission model replacing the old allowlist (dialog, fs scoped
                        to app data dir + configured mods folder, shell open, updater).
  src/
    main.rs           — slim entrypoint (calls lib.rs::run())
    lib.rs            — Builder setup: register plugins, open/init SQLite connection as managed
                        state, run schema.rs on startup, invoke_handler (empty/placeholder command
                        for now to confirm IPC wiring)
    db/
      mod.rs           — connection open (rusqlite, bundled), PRAGMA foreign_keys=ON, migration-on-startup runner
      schema.rs        — the CREATE TABLE statements above, run once via execute_batch
  _legacy/main.rs.reference  — old main.rs, archived per above (already done)
  definitions/zzz.toml       — kept (already done — genshin/hsr/wuwa removed)
```

DB file location: Tauri's app-data dir (e.g. `%APPDATA%/com.khawpunkp.gmm/gmm.db`), not a repo-relative path — matches the old app's convention of a per-install SQLite file.

## Verification

1. `npm install` in `C:\Users\kp\Desktop\workspace\GMM` — confirm clean install with the new package.json (Vue3/TS/Pinia/vue-router, no React deps left).
2. `npm run tauri dev` — confirm the Rust side compiles (`cargo build` via Tauri CLI) and a window opens titled "GMM" at 1400x800, showing the new sidebar + placeholder pages, styled with the ported dark theme (not the default Tauri/Vite splash).
3. Click through the 5 placeholder routes (Dashboard/Agents/Agent detail/Presets/Settings) via the sidebar — confirm client-side routing works and the layout shell renders consistently on each.
4. Inspect the created SQLite file (app-data dir) with a SQLite browser or `sqlite3 <path> ".tables"` — confirm all 8 tables exist with the columns above and no errors on the schema `execute_batch`.
5. `cargo check` in `src-tauri/` — confirm no dead-code warnings pointing at anything under `_legacy/` (it must not be part of the compiled crate).

## Progress so far

- [x] Old React `src/` removed (git rm)
- [x] Old `main.rs` archived to `src-tauri/_legacy/main.rs.reference` (git mv)
- [x] `genshin.toml`/`hsr.toml`/`wuwa.toml` removed, `zzz.toml` kept
- [x] New frontend scaffold (package.json, vite.config.ts, tsconfig.json, index.html, src/*)
- [x] New Tauri v2 backend scaffold (Cargo.toml, tauri.conf.json, capabilities, src-tauri/src/*)
- [x] `npm install` + `npm run tauri dev` verification — `cargo check` clean, `npm audit` 0 vulnerabilities,
      app compiled and launched (gmm.exe running), SQLite db created at `%APPDATA%/com.khawpunkp.gmm/gmm.db`
      with all 8 tables + correct columns confirmed via Python's sqlite3 module. Visual check (sidebar,
      5 routes, dark theme rendering) NOT done by Claude — left the dev window open for the user to eyeball.

## Known gaps to revisit in a later phase

- `capabilities/default.json` fs scope is currently `$APPDATA/**` + `$APPLOCALDATA/**` only — narrower than
  the plan's "app data dir + configured mods folder" because the mods-folder picker doesn't exist until a
  later phase. Extend the scope (or add it dynamically via the fs plugin's Rust API once a folder is chosen)
  when that UI lands.
- `shell:allow-execute` (for actually launching the game) was intentionally left out this phase — only
  `shell:allow-open` is granted. Add the scoped execute permission when the launcher UI is built.

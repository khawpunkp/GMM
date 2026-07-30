# GMM Rebuild — Phase 4: Mod Management UI

## Context

Phases 1-3 done, committed, pushed. This is Phase 4 per `REBUILD_NOTES.md`: "Mod management UI:
cards, enable/disable, presets, import modal, keybinds popup, launcher" — REBUILD_NOTES' own estimate
(150-220K) makes this the single biggest phase, and it bundles several genuinely independent features.
Before writing anything, I read the relevant legacy code to understand what's being ported:

- **Enable/disable** (`toggle_asset_enabled`, line 1976): checks which of the enabled/disabled path
  variants currently exists on disk, renames to the other. Simpler in the new schema than the old —
  `mods.folder_name` is globally unique now (no `(entity_id, folder_name)` composite needed).
- **Presets are full-system-state snapshots, not subsets** (`create_preset`/`apply_preset`, lines
  3677/3806) — this surprised me enough to flag explicitly. Creating a preset records **every mod's
  current enabled/disabled state**, not just a chosen subset. Applying a preset walks all its recorded
  mods and renames whichever ones don't match the recorded state. `preset_mods` (from Phase 1's schema)
  already matches this shape (`preset_id, mod_id, is_enabled`).
- **Keybinds popup** (`get_ini_keybinds`, line 4283): finds the mod's `.ini` file(s) via the same
  dual-path enabled/disabled check, then scans line-by-line for a `; Constants` comment marker,
  and inside `[Key...]` sections after that marker, collects `key = value` lines as `{title, key}`
  pairs. Old code also had `find_asset_ini_paths` (line 696) doing the dual-path INI lookup — needs
  porting once, reused for this and any future skin-toggle work (Phase 7).
- **Launcher** (`launch_executable`, line 1572): spawns the configured game executable via Tauri's
  shell command API, streaming stdout/stderr, with a specific handled case for Windows error 740
  (needs elevation). Maps directly onto `tauri-plugin-shell` (already a dependency since Phase 1) —
  this is exactly the `shell:allow-execute` capability gap Phase 1 flagged as a known TODO.

## Open question: stage this phase or do it all in one pass?

This phase bundles four largely-independent features (mod browsing/toggle, presets, import modal,
keybinds+launcher). Proposing to split the *implementation* into stages while still calling it all
"Phase 4" per REBUILD_NOTES' numbering — each stage gets its own commit rather than one giant diff:

- **4a**: mod cards + enable/disable (the core browsing experience — needed before anything else here
  is useful)
- **4b**: presets (create/list/apply/delete/favorite)
- **4c**: import modal (wires up Phase 3's `analyze_archive`/`import_archive` to an actual UI)
- **4d**: keybinds popup + launcher

Let me know if you'd rather I just plan+build the whole thing in one continuous pass instead.

## Frontend structure — proposal, flagging for your review

1. **`pages/agents/[slug].vue`**: gains a "Mods" section below the existing edit form, showing that
   agent's mods as cards with an enable/disable toggle. Reuses the same page rather than adding a new
   route, since mod browsing is naturally scoped to "this agent's mods."
2. **`pages/index.vue` (Dashboard, currently a placeholder)**: becomes the home for uncategorized mods
   (agent_id AND category_id both NULL) plus category-bucket mods (UI/enemies/etc.), and — since a
   mod manager's launch button is normally front-and-center, not buried in Settings — the **Launch
   Game** button lives here, not on the Settings page. Settings only holds the configuration (game
   executable path picker), same pattern as the mods-folder picker from Phase 3.
3. **`pages/presets.vue`** (currently a placeholder): real preset list — create/apply/delete/favorite.
4. **New shared components**: `ModCard.vue` (thumbnail, name, enable toggle, edit/delete/open-folder/
   keybinds buttons), `ModEditModal.vue` or inline edit (reusing the `AgentForm.vue` pattern — a form,
   not literally a browser `<dialog>`, consistent with how Add/Edit Agent works), `ImportModal.vue`
   (archive analysis review before confirming import), `KeybindsPopup.vue`.

## Backend: new commands

**Mods (4a)**
- `list_mods(agent_id?, category_id?, category_item_id?) -> Vec<ModWithState>` — joins `mods` with a
  live-computed `is_enabled` (checked on disk, same dual-path logic as toggle/apply — never stored,
  consistent with the existing convention).
- `toggle_mod_enabled(mod_id) -> bool` — the rename port described above.
- `update_mod_info(mod_id, input) -> ModWithState` — name/description/author/image edit.
- `delete_mod(mod_id)` — removes the folder from disk *and* the DB row (needs a confirmation in the UI
  given it's destructive — no undo).
- `open_mod_folder(mod_id)` — reveals the mod's folder in the OS file explorer (`shell:allow-open`,
  already granted since Phase 1).

**Presets (4b)**
- `create_preset(name) -> Preset` — snapshots every mod's current on/off state.
- `list_presets() -> Vec<Preset>`, `delete_preset(id)`, `toggle_preset_favorite(id, is_favorite)`.
- `apply_preset(id)` — renames whatever's out of sync with the snapshot; emits progress events same
  pattern as the Phase 3 scanner (`preset-apply-progress`/`-complete`/`-error`).
- `overwrite_preset(id)` — re-snapshots an existing preset with the current state.

**Import (4c)** — no new backend, wires the frontend to Phase 3's already-built `analyze_archive`/
`import_archive`.

**Keybinds + launcher (4d)**
- `get_mod_keybinds(mod_id) -> Vec<KeybindInfo>` — ports `find_asset_ini_paths` + the `; Constants`
  scan described above.
- `launch_game() -> Result<(), String>` — reads a new `game_executable_path` setting (same
  `get_setting`/`set_setting` infra from Phase 3), spawns it via `tauri_plugin_shell`, surfaces the
  Windows-740-needs-elevation case as a specific error message like the old app did.

## Capability change needed

`capabilities/default.json` needs a `shell:allow-execute` entry scoped to `cmd: "{0}"` (execute
whatever absolute path is passed in) — mirrors the old v1 allowlist's `execute-any-file` entry. This
is deliberately permissive (not a fixed allowlist of specific programs) because the whole point is
launching whatever game executable the user points at via the Settings file picker, same reasoning as
the already-permissive fs scope from Phase 1. Extends the "known gaps" flagged back in Phase 1.

## Explicitly out of scope for Phase 4

- **Mod grouping** (`mod_groups`/`mod_group_members`) — REBUILD_NOTES scopes this as its own Phase 5,
  not part of "cards, enable/disable, presets, import modal, keybinds popup, launcher." Not touching
  those tables this phase even though they already exist in the schema.
- **Skin-toggle memory** (Phase 7) — `get_mod_keybinds` is read-only display; writing a keybind choice
  back into the `.ini` before launch is explicitly flagged in REBUILD_NOTES as needing its own research
  spike, not bundled in here.

## Unplanned change: switched from `tauri-plugin-shell`'s `open()` to `tauri-plugin-opener`

`open_mod_folder` originally used `tauri_plugin_shell::ShellExt`'s `.open()` (same as Phase 1's
`shell:allow-open` capability) — `cargo check` surfaced a real deprecation warning: that method is
deprecated in favor of `tauri-plugin-opener`, the officially recommended plugin specifically for
"open a file/folder/URL with the OS default handler." Added `tauri-plugin-opener` as a dependency,
registered it in `lib.rs`, swapped the capability entry from `shell:allow-open` to `opener:default`
(this is the same `opener:default` permission Phase 1 had originally stubbed and then deliberately
removed — turns out that removal was premature). `tauri-plugin-shell` stays a dependency for 4d's
launcher, which needs actual process-spawning (`shell:allow-execute`), a genuinely different
capability that `opener` doesn't cover.

## 4a verification

**Automated** (`cargo test`, both pass):
- `toggle_flips_enabled_state_and_renames_on_disk` — toggles a synthetic mod folder twice, asserts
  the rename direction and `is_mod_enabled` state each time.
- `toggle_errors_when_folder_missing` — confirms a missing folder surfaces a clear error instead of
  panicking.

`cargo check` and `vue-tsc -b` both clean.

**Manual, flagged for you**: actually clicking through the new Mods section on an agent's page. One
real thing I found while checking the live app's state (not touching it myself — this is your actual
game config, not test data): `mods_folder_path` is currently set to
`...\XXMI Launcher\ZZMI\Mods\DISABLED_Astra Shining Eridu` — that's a single mod's own folder, one
level too deep. The scanner treats whatever path you give it as the *parent* containing all mod
folders, so pointed at one specific mod's folder it finds zero mods inside it (there's nothing to
recurse into). You'll want to repoint it at the `...\ZZMI\Mods` folder itself via the Settings page,
then Scan Now, to see real mod cards on the agent pages.

## 4b verification

**Automated** (`cargo test`, all 5 pass across the whole crate now):
- `apply_preset_restores_snapshotted_state` — creates a preset (snapshotting 2 synthetic mods, both
  enabled), disables one on disk, applies the preset, confirms it's toggled back to match the
  snapshot and confirms the untouched mod stays untouched.
- `overwrite_preset_replaces_the_snapshot` — changes disk state, overwrites the preset, confirms
  re-applying afterward is a no-op against the new snapshot.

`cargo check` and `vue-tsc -b` both clean.

**Added beyond the original plan doc**: wired the Sidebar's `preset-section` (a static placeholder
since Phase 1) to show real favorite presets (top 3, matching the old app's `get_favorite_presets`
convention) instead of leaving it as dead UI once real presets existed to show. Also fixed
`types/index.ts`'s `Preset` interface — it had a speculative `mods: PresetModEntry[]` field from
Phase 1 that never matched what the backend actually returns; removed it along with the now-unused
`PresetModEntry` type.

**Manual, flagged for you**: actually clicking through create/apply/overwrite/delete/favorite and
watching the live apply-progress status line.

## Progress so far

- [x] Staging decision confirmed: 4a → 4b → 4c → 4d, each its own commit
- [x] 4a: mod cards + enable/disable
- [x] 4b: presets
- [ ] 4c: import modal
- [ ] 4d: keybinds popup + launcher

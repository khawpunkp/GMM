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

## Capability change needed — this turned out to be wrong, corrected during 4d

This section originally said `capabilities/default.json` needs a `shell:allow-execute` entry scoped
to `cmd: "{0}"`, mirroring v1's `execute-any-file` wildcard. Before implementing, I read the actual
installed `tauri-plugin-shell@2.3.5` crate source (`scope_entry.rs`/`scope.rs`/`lib.rs`) rather than
trust that assumption, and it's flatly wrong for v2: each scope entry's `cmd` field deserializes as a
**fixed `PathBuf`** — there's no `"{0}"` wildcard mechanism anymore. A static capabilities entry can't
pre-declare a path the user only picks at runtime via the Settings file dialog, so the v1-style
approach was never going to work here at all.

The actual fix needed **no capability change whatsoever**: `Shell::command()` (the plugin's Rust-side
API, called from my own custom `launch_game` command) calls `Command::new(program)` directly with
**zero scope validation** — the named-scope lookup (`ShellScope::prepare`) only guards the plugin's
own JS-invokable `execute` IPC command, not Rust-side calls to `Shell::command()`. Same "bypass the
IPC-facing capability scope from Rust code" pattern already used for `read_image_as_data_url` (Phase 2)
and the scanner's/archive importer's direct filesystem access (Phase 3) — capability scopes only gate
the JS→Rust IPC boundary, not code I write on the Rust side that never crosses it.

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
launcher, which needs actual process-spawning via the plugin's Rust API — see the corrected
"Capability change needed" section above for why that ended up needing no capability entry at all.

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

## 4c: import modal — scoped narrower than the original plan text

Built `ImportModal.vue`: pick a `.zip`/`.7z`/`.rar` via the dialog plugin, `analyze_archive`, a review
form (name/description/author, a root picker if the archive has multiple likely mod roots), then
`import_archive` on submit. Entry point is a new "+ Import Mod" button in the agent detail page's
Mods section header — no new backend, exactly as planned (wires up Phase 3's already-built commands).

**Deliberately narrower than it could be**: the modal always imports into whichever agent's page you
opened it from (`agentId` prop), even if the archive's own analysis detects a *different* target (or
a category instead of an agent). A fully general import entry point — open it from anywhere, let the
analysis's own agent/category detection freely apply, with a search-across-51-agents-and-5-categories
picker to override — would need a real picker component I didn't build this pass. What's here covers
the concrete, common case (you're looking at a character's page, you want to add a mod for them)
without that extra UI surface. Flagging as a real gap, not a silent omission — worth a follow-up if a
global "Import Mod" entry point on the Dashboard or Agents list turns out to matter in practice.

**Verification**: `vue-tsc -b` clean (no backend changes this stage, so no `cargo` run needed). Fetched
the updated agent page and the new component from the dev server to confirm they transform without
errors. **Not tested against a real archive** — same gap Phase 3 flagged for `analyze_archive`/
`import_archive` themselves; try a real `.zip` mod through this modal before trusting the path.

## 4d verification

**Automated** (`cargo test`, all 8 pass across the whole crate now):
- `parses_keybinds_after_constants_marker` / `ignores_key_sections_before_constants_marker` /
  `returns_empty_when_no_constants_marker` — the keybinds parser is a pure `&str -> Vec<KeybindInfo>`
  function (`parse_keybinds_from_ini`), split out specifically so it's testable without touching the
  filesystem. Covers the actual risky logic (the `; Constants` marker + `[Key...]` section scan).

`cargo check` and `vue-tsc -b` both clean. `launch_game` itself isn't unit-tested — it's inherently
OS-process-spawning, not meaningfully testable without mocking the shell plugin, and I don't have a
real game executable to test it against live. Same category of gap as Phase 3's untested archive
extraction — flagging rather than silently skipping.

**Added beyond the original plan**: built out `pages/index.vue` (the Dashboard, a placeholder since
Phase 1) with a Launch Game button and basic mod-count stats (total/enabled/uncategorized), since the
launcher needed *somewhere* to live and the plan itself argued Settings shouldn't be it. Settings
gained a second section (Game Executable path picker) alongside the existing Mods Folder one.
`ModCard.vue` gained a fourth icon button (keybinds) alongside edit/open-folder/delete.

**Manual, flagged for you**: clicking Launch Game against a real configured executable, and opening
Keybinds on a mod that actually has a `; Constants` section with real `[Key...]` bindings.

## Progress so far

- [x] Staging decision confirmed: 4a → 4b → 4c → 4d, each its own commit
- [x] 4a: mod cards + enable/disable
- [x] 4b: presets
- [x] 4c: import modal (agent-scoped only — see gap noted above)
- [x] 4d: keybinds popup + launcher — **Phase 4 complete**

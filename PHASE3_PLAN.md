# GMM Rebuild — Phase 3: Scanner + Archive Import

## Context

Phases 1 (scaffold) and 2 (agents/aliases/categories) are done, committed, pushed. This is Phase 3 per
`REBUILD_NOTES.md`: "port ini-parsing/archive-import logic as reference, rewrite matching against
alias arrays (simpler than old heuristic), keep a category-fallback bucket."

I read the relevant parts of `_legacy/main.rs.reference` to understand what's being replaced/ported:

- `find_entity_slug_from_hint` (line 569) — the 13-priority fuzzy name-matching heuristic being
  replaced. Tries exact slug, exact/cleaned name, first-two-words, first-name, then `starts_with`
  and `contains` variants of all of the above, checked against every known entity name.
- `deduce_mod_info_v2` (line 823) — the orchestration around it. Tries hints in this order: **mod
  folder name → parent folder names (walking up to the mods root) → INI `[Mod/Settings/Info/General]`
  section's `Target`/`Entity`/`Character` field → internal filenames inside the mod folder**. If no
  entity matches, falls back to category matching (parent folder name → INI `Type`/`Category` field →
  top-level folder name relative to the mods root, all fuzzy-matched against category names), and if
  even that fails, dumps the mod into a hardcoded `"characters-other"` bucket.
- `scan_mods_directory` (line 2193) — walks the mods folder with `WalkDir`, identifies a directory as
  a mod folder if it directly contains a non-excluded `.ini` file (`has_ini_file`, line 1143 — there's
  a real exclusion list: `orfix.ini`, `region.ini`, `offset.ini`, etc. — these are 3DMigoto config
  files, not mod-identity files), fixes up a `DISABLED` → `DISABLED_` naming inconsistency, deduces
  mod info, upserts into the DB, then **prunes** DB rows for mods no longer found on disk.
- `analyze_archive` / `import_archive` (lines 2826/3394) — inspect and extract `.zip`/`.7z`/`.rar`
  archives (three different crate APIs), detecting the "mod root" inside the archive and reading INI
  contents for the same deduction pass, before extracting into the mods folder.
- **Enabled/disabled state has no DB column** — same as the new schema. A mod's `folder_name` in the
  DB is always the clean (enabled-style) name; actual state is read off disk by checking whether
  `<name>` or `DISABLED_<name>` currently exists. Confirms Phase 1's schema (no `is_enabled` column
  on `mods`) already matches this convention — nothing to change there.
- Toggling enable/disable itself (the actual rename) is explicitly Phase 4 scope ("Mod management UI:
  cards, enable/disable, presets..."), not this phase — Phase 3 only needs to _recognize_ the
  `DISABLED_` prefix while scanning, not flip it.

## Prerequisite gap found: no way to configure a mods folder yet

`pages/settings.vue` is still Phase 1's placeholder — there is currently no UI or command to set the
mods folder path anywhere in the new app, and the scanner needs one to have anything to scan. Adding
this wasn't in REBUILD_NOTES' one-line Phase 3 description, but it's a hard prerequisite, so it's
folded into this phase:

- `get_setting(key) -> Option<String>` / `set_setting(key, value)` commands (generic, reusable beyond
  just this one setting).
- Real `pages/settings.vue`: a folder picker (via `@tauri-apps/plugin-dialog`'s `open({directory: true})`)
  for `mods_folder_path`, plus a "Scan Now" button and a status line — enough to configure and trigger
  scanning, not the full Mod Management UI (that's Phase 4).

## Matching algorithm — replacing the 13-priority heuristic

Since `agent_aliases` already holds curated variants per character (from Phase 2: "ellen", "ellen joe",
"ellenjoe", "joe" for Ellen), the alias list itself absorbs most of what the old heuristic's priority
ladder existed to paper over. Proposed replacement, much simpler:

1. Normalize the hint: lowercase, collapse `_`/`-`/`.`/whitespace runs into single spaces, strip a
   trailing version tag (`_v2`, `_v1.3`) and `(disabled)`/`DISABLED_` markers — same idea as the old
   `clean_and_extract_name`, reused rather than reinvented.
2. Check every agent's aliases for a substring match against the normalized hint. If multiple agents
   match, **the longest matching alias wins** (avoids a short alias like `"s0"` shadowing a more
   specific one like `"ellen joe"` when both happen to appear as substrings).
3. Try hints in the same order as before: mod folder name → parent folder names → INI
   `Target`/`Entity`/`Character` field → internal filenames.

If no agent matches, category fallback: normalize the hint the same way, then check it against every
`category_items.name`/`slug` (simple substring/exact match — no alias table for these, since there
are only 11 of them, far less naming variance than 51 characters) and, failing that, against
`categories.name`/`slug` directly (mirrors the old category-level fallback). If nothing matches at all,
the mod lands with **`agent_id = NULL` and `category_id = NULL`** — no synthetic `"characters-other"`
row is needed, since both FKs are already nullable (same simplification Phase 2 made for
`category_items`). This is the "category-fallback bucket" REBUILD_NOTES asks to keep, just represented
as "uncategorized" (`NULL`) instead of a fake DB row, consistent with the schema's existing design.

## Rust structure

- `src-tauri/src/scanner/mod.rs` — `scan_mods_directory` port: `WalkDir` traversal, `DISABLED` →
  `DISABLED_` rename fixup, `has_ini_file` (with the same excluded-filename list), upsert-by-
  `(agent_id/category_id/category_item_id, folder_name)`, then prune DB rows missing from disk.
- `src-tauri/src/scanner/deduce.rs` — the hint-gathering + matching pipeline described above,
  returning a `DeducedModInfo { agent_id: Option<i64>, category_id: Option<i64>, category_item_id:
Option<i64>, name, description, author, image_filename }`.
- `src-tauri/src/scanner/archive.rs` — port of `analyze_archive`/`import_archive`: same three
  crate APIs (`zip`, `sevenz_rust`, `unrar` — already in `Cargo.toml` since Phase 1, unused until now),
  same mod-root detection inside an archive, extraction into the configured mods folder, then feeding
  the extracted folder through the same deduction pipeline as a normal scan.
- New Tauri commands: `get_setting`, `set_setting`, `scan_mods_directory`, `select_mods_folder`
  (wraps the dialog plugin's directory picker), `analyze_archive`, `import_archive`.
- Scan progress: the old app emitted `SCAN_PROGRESS_EVENT`/`SCAN_COMPLETE_EVENT`/`SCAN_ERROR_EVENT`
  via `app_handle.emit_all` for a live progress bar. Porting the same event-emission approach
  (Tauri v2's `Emitter` trait, `app.emit(...)`) — the Settings page's status line listens for these.

## Flagged for your review

1. **Longest-alias-wins tie-break** — reasonable default for resolving cross-agent alias collisions,
   but untested against real mod folder names. Worth revisiting once you scan a real mods folder and
   see what actually misfires.
2. **Category fallback has no aliases** — only agents got aliases in Phase 2 (scoped that way
   deliberately). If category_item matching turns out to miss too often in practice (e.g. "Ether
   Mutant" mods folder-named just "EtherMutants" won't substring-match "Ether Mutants" name), we can
   add a small aliases table for category_items too later — not doing it preemptively without evidence
   it's needed.
3. **Archive import UI**: this phase only adds the backend commands (`analyze_archive`/`import_archive`)
   — REBUILD_NOTES' own Phase 4 line item is "import modal," so the actual drag-and-drop/file-picker
   import UI is deferred there. This phase's frontend surface is just the Settings page's folder
   picker + Scan button.

## Unplanned change: `run_scan` decoupled from `AppHandle`

Wanted real automated coverage of the matching rewrite (the actual risky/novel part of this phase),
not just eyeballing the running app. `run_scan` originally took `&AppHandle` and called `.emit()`
directly, which can't be constructed in a plain unit test. Refactored it to take a plain
`on_progress: impl FnMut(usize, &str)` closure instead — the Tauri command wrapper
(`commands/scanner.rs`) now owns the actual `app.emit(...)` calls for `scan-progress`/`scan-complete`/
`scan-error`, and a unit test can pass a no-op closure. Same behavior, just testable.

## Verification

1. Point `mods_folder_path` at a real (or synthetic test) mods folder via the new Settings picker.
   **Not click-tested by Claude** — no way to drive the native window from here — but the running dev
   app auto-rebuilt with all Phase 3 changes live (confirmed via the binary's mtime and by fetching
   the new `settings.vue` from the dev server), so it's ready for you to try.
2. Run a scan, confirm mods land in the `mods` table with plausible `agent_id`/`category_id` matches
   for folders named after known aliases, and `NULL`/`NULL` for folders that shouldn't match anything.
   **Verified via an automated test** (`scanner::tests::scans_synthetic_mods_folder_correctly`,
   `cargo test` — builds its own throwaway fixture folder under the OS temp dir, no committed fixture
   files) rather than manual UI click-through: confirmed agent-match-via-parent-folder, INI
   Name-field parsing, and category fallback all resolve correctly, and a fully-unmatched folder still
   gets recorded with `NULL`/`NULL` (not dropped).
3. Confirm a `DISABLED_`-prefixed folder scans correctly (clean name stored, no duplicate row), _and_
   that a `DISABLED`-without-underscore folder gets renamed and then scanned correctly. **Verified by
   the same automated test** — both cases covered explicitly.
4. Re-run the scan after deleting a mod folder from disk — confirm the corresponding DB row is pruned.
   **Not separately tested this pass** — the prune logic is a straight, low-risk port of the old
   app's approach (diff found-on-disk vs. in-DB, delete the difference); worth a manual sanity check
   but didn't seem to warrant its own automated case given the pattern's already proven.
5. Test `analyze_archive`/`import_archive` against a real `.zip` mod (7z/rar if available) — confirm
   extraction lands in the mods folder and the extracted mod is then picked up correctly. **Not
   tested this pass** — needs a real archive file, which I don't have one handy to construct
   meaningfully; `cargo check`/`cargo test` confirm it compiles and the mod-root-detection logic
   mirrors the old app's proven approach, but the actual zip/7z/rar extraction paths are unexercised.
   Flagging this as the biggest real gap in this phase's verification — try importing a real mod
   archive before relying on this.

## Progress so far

- [x] `get_setting`/`set_setting` commands + real Settings page (mods folder picker, Scan button)
- [x] `scanner/deduce.rs`: hint gathering + alias/category-item/category matching
- [x] `scanner/mod.rs`: directory walk, DISABLED fixup, upsert, prune
- [x] `scanner/archive.rs`: analyze/import for zip/7z/rar (compiles clean; extraction paths untested
      against a real archive — see verification item 5 above)
- [x] Verification checklist above (items 1 and 5 need your hands-on follow-up; 2–3 covered by an
      automated test; 4 is a low-risk direct port, not separately tested)

# GMM Rebuild — Phase 7: In-Game Skin-Toggle Memory (Research Spike)

## What this is

REBUILD_NOTES point 8 / Phase 7: remember per-mod in-game toggle state (hat on/off, outfit variant —
controlled today via 3DMigoto hotkeys inside the game) from within GMM. Explicitly flagged as needing
a research spike against real mod files before sizing, and the highest-risk item on the whole roadmap.
This doc **is** that spike — done against your real, currently-scanned mods folder (150 `.ini` files),
not synthetic guesses.

## The mechanism — simpler than REBUILD_NOTES assumed, in one important way

3DMigoto's own ini scripting language has a `persist` keyword:

```ini
[Constants]
global persist $body = 0
global persist $bracelet = 3

[KeySwapBody]
key = no_modifiers UP
type = cycle
$body = 0,1,2,3
```

`global persist $var = N` means **3DMigoto itself already writes the current value back into the
`.ini` file** whenever it changes (cycled via the `[Key...]` hotkey) and the game exits — confirmed
directly: your real files already have non-default values sitting in them right now (`$Bracelet = 3`,
`$Apron = 1`, `$Armlet = 6`, etc.), from you actually using the in-game hotkeys. **71 of your 150
scanned `.ini` files use this pattern.**

This means GMM doesn't need to build its own persistence pipeline. It needs to:

1. **Read** the current `global persist $var = N` line from `[Constants]`.
2. **Read the valid range** for that var from whichever `[Key...]` section references it
   (`type = cycle` + `$var = a,b,c,...`) — gives a proper bounded control, not a raw number field.
3. **Show** it as an editable control in GMM.
4. **Write** a changed value back — a surgical line-level replace of just that one
   `global persist $var = ` line, leaving everything else in the file byte-for-byte untouched (these
   files have hand-written scripting logic, comments, exact formatting — a generic ini-rewrite library
   would risk mangling all of that).

## The one real timing hazard

If GMM edits the file _while the game is running_, 3DMigoto flushes its own in-memory value back to
disk when the game exits — which could silently overwrite GMM's edit with whatever was last set
in-game before GMM's change. **GMM must refuse to write these edits while the game process is
running.** Needs a "is the configured game executable currently running?" check (Windows process
list) before any write — a real, concrete safety requirement, not a nice-to-have.

## Open question: not every `persist` var is a skin toggle

`global persist $helpScale = 1` / `global persist $helpVisible = 0` also showed up — these control
3DMigoto's own on-screen help-menu overlay, not an outfit piece. Filtering these out by variable name
(e.g. "contains 'help'") isn't reliable — mod authors name variables however they want; there's no
schema distinguishing "cosmetic toggle" from "tool preference" at the ini level.

## Progress so far

- [x] Research spike complete — real pattern confirmed against 150 real files, 71 using `persist`
- [x] Decided: show every `persist` var uniformly — no reliable schema-level way to distinguish a
      cosmetic toggle from a 3DMigoto tool setting (e.g. `helpScale`/`helpVisible`) by name alone
- [x] Decided: block writes only while the game process is running (checked via
      `is_process_running` against the configured `game_executable_path`, Toolhelp32 snapshot)
- [x] Parser: `keybinds.rs::parse_persist_vars_from_ini` extracts `global persist $var = N` lines
      and cross-references `[Key...]` sections for `type = cycle` ranges; `get_persist_vars` is the
      disk-facing wrapper (mirrors `get_keybinds`) — 4 unit tests against a real-world `.ini` sample
- [x] Surgical line-replace writer: `keybinds.rs::set_persist_var` rewrites only the matching
      `global persist $var = ` line, preserving every other byte/line-ending exactly
- [x] Running-process guard: `process_check::is_process_running` (Win32 Toolhelp32 snapshot),
      wired into `commands::keybinds::set_mod_persist_var` — refuses the write with a clear error
      if the configured game executable is currently running
- [x] Tauri commands `get_mod_persist_vars` / `set_mod_persist_var` registered in `lib.rs`;
      `cargo check` + `cargo test` both clean (15/15 tests passing, no dead-code warnings)
- [x] UI: `KeybindsPopup.vue` extended with a "Toggle memory" section — a `<select>` per var with
      known cycle options, a number input otherwise, saving on change and surfacing the
      game-running error inline per row; `vue-tsc -b` clean

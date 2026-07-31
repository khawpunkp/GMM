# GMM Rebuild — Phase 9: Remove Presets + Dashboard, pin Quick Launch / Import Mod

## What this is

Not in the original roadmap — ad-hoc follow-up. User asked to remove the Presets feature and the
Dashboard page outright, after Phase 8 added the 5 category tabs.

## Confirmed design (via AskUserQuestion)

1. **Full delete**, not just hidden from nav — DB tables, Rust commands/modules, frontend
   pages/stores/components, sidebar nav items all removed.
2. **Agents is now the landing page.** `/` redirects to `/agents` (added as an explicit route in
   `main.ts` since deleting `pages/index.vue` means unplugin-vue-router no longer generates a `/`
   route on its own).
3. **Quick Launch + Import Mod are now pinned in the Sidebar**, matching the old app's screenshot
   shown earlier in this session — always visible, not tied to any one page.

## What changed

**Backend:**

- Deleted `presets.rs`, `commands/presets.rs`; removed `mod presets;` / `pub mod presets;` /
  `commands::presets::*` handler registrations; removed the `Preset` model
- `db/schema.rs`: dropped the `presets` / `preset_mods` table definitions
- `db/mod.rs`: new one-time flag-gated migration `migrate_drop_presets` — `DROP TABLE IF EXISTS`
  for both tables on existing installs (same pattern as Phase 5's group-members migration), since
  `CREATE TABLE IF NOT EXISTS` alone never removes tables from real user DBs

**Frontend:**

- Deleted `pages/presets.vue`, `stores/presets.ts`, `pages/index.vue` (Dashboard), the `Preset` TS
  type, and the Sidebar's favorite-presets block
- `Sidebar.vue` rebuilt: pinned "Quick Launch" button (ported from the old Dashboard's launch
  logic) and a pinned "Import Mod" button at the top, above the nav list; nav list itself now just
  Agents / 5 categories / Settings
- `ImportModal.vue` gained a destination picker (one select covering every Agent + the 5
  categories, same shape as `ModEditModal`'s recategorize dropdown) that only appears when the
  modal is opened with **no** fixed `agentId`/`categoryId` prop — i.e. only for the sidebar's
  global button. Per-page "+ Import Mod" buttons (Agent page, category pages) keep passing a fixed
  target and never see the picker.

## Verification

- `cargo check` + `cargo test` (18/18 passing — 2 fewer than Phase 8's 20, matching the removed
  preset unit tests) + `vue-tsc -b` all clean

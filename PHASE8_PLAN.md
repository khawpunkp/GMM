# GMM Rebuild — Phase 8: Category Browsing (NPCs / Enemies / Weapons / Objects / UI)

## What this is

Not in the original 7-phase roadmap — new ask, prompted by a screenshot of the old app's sidebar
(Home / Characters / NPCs / Objects / Enemies / Weapons / UI, plus pinned Quick Launch + Import Mod
buttons). The rebuild currently only has a nav item for Agents (Characters); the other 5 category
tabs from the old app don't exist yet, even though the DB schema/seed already fully supports them
(`categories` + `category_items` tables, seeded from `definitions/zzz.toml`, `mods.category_id` /
`category_item_id` columns, and `list_mods`/`import_archive` already accept both — this is purely a
missing frontend feature, no schema change needed).

## Confirmed design (via AskUserQuestion)

1. **No sub-category browsing for the 5 non-Characters tabs.** Characters keeps its existing
   two-level structure (agent grid → agent detail page with that agent's mods). NPCs/Enemies/
   Weapons/Objects/UI each go **straight to a flat mod-card grid** for that category — no
   item-picker grid in between, even though `category_items` rows exist in the DB (Enemies/Objects/UI
   have a few seeded; NPCs/Weapons currently have zero). Those `category_item` rows stay as
   scanner-matching data, not a browsing UI concept.
2. **No user-facing "add category item" flow.** Only Agents keep "+ Add Agent". NPCs/Weapons having
   zero seeded items is fine — mods imported into those categories just have `category_item_id =
   NULL` and show up in the flat grid regardless.
3. **Corrected:** each category gets a real, seeded **"Other"** `category_item` (e.g. "Other NPCs",
   "Other Enemies", ...) — a catch-all row, not just a `NULL` `category_item_id`. Anything the
   scanner/import can't match to a specific item is explicitly filed under that category's "Other"
   item rather than left unattributed. It still renders in the same flat mod-card grid as everything
   else in that category (no separate "Other" page/sub-view) — this only changes what the mod is
   *tagged* as internally, not how it's browsed.
4. **"Agents" label stays as-is** (not renamed to "Characters").
5. **Confirmed: mods can be recategorized after import.** The mod edit modal gets a
   category/agent-reassignment dropdown covering all 6 destinations (a specific Agent, or one of the
   5 categories — landing a mod on a category with no specific item picked assigns it to that
   category's "Other" item).

## What already exists (no change needed)

- `categories` / `category_items` tables + seed sync (`db/seed.rs`), already populated from
  `definitions/zzz.toml`'s `[npcs]`/`[enemies]`/`[weapons]`/`[objects]`/`[ui]` sections
- `mods::list_mods(conn, path, agent_id, category_id, category_item_id)` — passing only
  `category_id` (others `None`) already returns every mod in that category, item or not
- `commands::mods::list_mods` Tauri command — already exposes all three filter params
- `import_archive`'s request shape already carries `categoryId`/`categoryItemId` (both nullable) —
  `ImportModal.vue` just currently hardcodes them to `null` and only accepts an `agentId` prop

## What's actually new

**Backend:**
- `db/seed.rs::sync_categories` — after syncing each category's TOML-defined items, ensure a
  permanent `{slug}-other` "Other" item exists for that category (seeded once, excluded from the
  existing prune-unused-items loop so it's never deleted even with no mods in it)
- `Category { id, name, slug }` model + `list_categories` Tauri command (nothing currently exposes
  the category id/slug list to the frontend — needed so the sidebar/pages can resolve "npcs" →
  category_id without hardcoding numeric ids)
- `update_mod_category(mod_id, agent_id: Option<i64>, category_id: Option<i64>, category_item_id:
  Option<i64>)` — new command + `mods::` function. Today `update_mod`/`ModInput` only ever touch
  name/description/author/image; there's no existing path to change a mod's agent/category
  assignment post-import at all. When `category_id` is set but `category_item_id` isn't, resolves to
  that category's "Other" item server-side — frontend never needs to know the "Other" item's id.
- `import_archive` gets the same "resolve to Other when categoryId is set but categoryItemId isn't"
  behavior, so `ImportModal.vue` can keep just passing `categoryId` for the 5 flat category pages.

**Frontend:**
- `stores/categories.ts` (fetch + cache the 5 non-agent categories and their ids)
- New page `pages/categories/[slug].vue` — one route, reused for all 5 tabs (category resolved from
  the route param via the categories store), mirroring the mod-grid half of `agents/[slug].vue`
  minus the agent form and minus grouping (grouping stays agent-scoped for now — not asked for here)
- `Sidebar.vue`: 5 new nav items (NPCs/Enemies/Weapons/Objects/UI → `/categories/npcs` etc.)
- `ImportModal.vue`: generalize the single `agentId` prop into a target union (`{ agentId }` or
  `{ categoryId }`) so category pages can import into themselves
- `ModEditModal.vue`: add the category/agent reassignment dropdown described above
- `ModCard.vue`: no change expected — already generic over any `ModWithState`

## Explicitly out of scope for this phase

- Quick Launch / Import Mod being "pinned" globally in the sidebar like the old app — that was a
  screenshot of the **old** app for context, not a request to change where those buttons live today
  (Launch Game stays on the Dashboard, Import Mod stays a per-page action). Not touching this unless
  asked separately.
- Mod grouping (virtual groups) on category pages — stays Agents-only, matches Phase 5 scope.

## Progress so far

- [x] Seed a permanent "Other" `category_item` per category (`db/seed.rs::sync_categories`,
      slug `{category}-other`, protected from the prune-unused loop)
- [x] `Category` model + `list_categories` command (`commands/categories.rs`)
- [x] `update_mod_category` command + backend function — resolves to "Other" server-side, and
      physically moves the mod's folder on disk (preserving enabled/DISABLED_ state) so
      `folder_name` never drifts from the new agent/category assignment; 5 new unit tests
- [x] `import_archive` resolves to "Other" server-side too (`mods::resolve_category_item_or_other`,
      shared by both call sites); `resolve_dest_subpath` promoted to `pub(crate)
      resolve_category_subpath` so import and recategorize agree on folder layout
- [x] `stores/categories.ts` + `Category` TS type; `stores/mods.ts` gained `fetchByCategory` /
      `updateCategory`
- [x] `pages/categories/[slug].vue` — one route reused for all 5 tabs, flat mod grid, no grouping
- [x] Sidebar nav items (NPCs/Enemies/Weapons/Objects/UI, each its own icon)
- [x] `ImportModal.vue` generalized target (`agentId?` / `categoryId?` props, both optional)
- [x] `ModEditModal.vue` reassignment dropdown — one select covering every Agent + the 5 categories,
      wired into both `agents/[slug].vue` and the new category page via a `recategorize` event
- [x] `cargo check` + `cargo test` (20/20 passing) + `vue-tsc -b` all clean

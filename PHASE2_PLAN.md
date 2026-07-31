# GMM Rebuild — Phase 2: Agents, Aliases, Categories

## Context

Phase 1 (scaffold) is done, verified, committed, and pushed (`c780492`). This is Phase 2 of 7 per
`REBUILD_NOTES.md`: seed the built-in ZZZ roster into the `agents`/`agent_aliases` tables, add the
Add/Edit Agent UI, and — per this session's scoping discussion — also seed the non-character
`categories`/`category_items` data from `zzz.toml`, since the source data is already sitting there.

Three scoping questions were resolved before writing this plan:

1. **Categories**: seed them now (not deferred to Phase 3), including a new `category_items` table —
   see schema change below.
2. **Aliases**: I author a best-effort alias list per character now (nicknames, no-space variants,
   obvious shorthand), clearly unverified, for you to review/correct. Low cost if wrong — a mod just
   falls into the uncategorized bucket instead of auto-matching.
3. **Built-in agents**: editable (description/aliases/image) through the Add/Edit UI, not deletable.

## Prior art found in `_legacy/main.rs.reference` — reused, not reinvented

The old app already solved "seed once + let the roster grow across future updates" via
`sync_definitions()` (line 1256) + a version gate in `initialize_database()` (line 1392):

- Parses `definitions/{game}.toml` into category → entity structs.
- **UPSERTs** categories and entities by slug (`ON CONFLICT(slug) DO UPDATE`), which preserves IDs —
  critical, since mods reference entities by ID.
- **Prunes** entities no longer present in the TOML, but only if they have zero mods attached
  (`has_assets` check) — never silently deletes user data.
- Gates the whole sync behind comparing `app_handle.package_info().version` against a
  `settings` row (`app_version`) — only re-parses/re-syncs on a version bump, not every startup.
- Every category gets a synthetic `"<slug>-other"` entity as a catch-all for unmatched mods.

Phase 2 ports this mechanism, split across two sync paths (agents vs. categories), since the new
schema splits what used to be one `entities` table into two concepts. One simplification falls out
naturally: **the synthetic "Other" entity hack is no longer needed** — `mods.agent_id` and
`mods.category_item_id` are already nullable, so "uncategorized within this bucket" is just `NULL`,
not a fake row.

## Schema changes (migration on top of Phase 1's schema.rs)

```sql
CREATE TABLE IF NOT EXISTS category_items (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id INTEGER NOT NULL,
    name        TEXT NOT NULL,
    slug        TEXT UNIQUE NOT NULL,
    description TEXT,
    base_image  TEXT,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE CASCADE
);
```

`mods` gains one column:

```sql
category_item_id INTEGER,  -- nullable; FOREIGN KEY REFERENCES category_items(id) ON DELETE SET NULL
```

Since there's no real user data yet (pre-release, Phase 1's dev DB is throwaway), this ships as a
straight edit to `schema.rs`'s `CREATE TABLE` statements rather than a real migration path — you'll
need to delete your local `%APPDATA%\com.khawpunkp.gmm\gmm.db` once after this lands (or I add a
one-line startup check that does it automatically for this transitional case, your call).

## Seed data: extend `zzz.toml` in place

Rather than a new file/format, add an `aliases = [...]` array to each `[characters]` entity (the only
new field needed) and leave the other category tables (`npcs`/`enemies`/`weapons`/`objects`/`ui`)
structurally unchanged — they already match what `category_items` needs.

Example edit:

```toml
{ name = "Ellen", slug = "ellen", description = "...", details = '...', base_image = "ellen_base.jpg",
  aliases = ["ellen", "ellen joe", "ellenjoe", "joe"] }
```

I'll write best-effort aliases for all 46 characters this way (flagged as unverified — happy to have
you spot-check a sample before this ships).

## Rust: split sync into two paths

`src-tauri/src/db/seed.rs` (new):

- `AgentDefinition { name, slug, description, details, base_image, #[serde(default)] aliases: Vec<String> }`
   - `CharacterCategoryDefinition { name, entities: Vec<AgentDefinition> }` — parsed by removing the
     `"characters"` key from the parsed `toml::Value` table before the generic pass.
- `CategoryItemDefinition` / `CategoryDefinition` (renamed from the old `EntityDefinition`) — same
  shape as before, parsed from whatever's left of the table after `"characters"` is removed.
- `sync_agents(tx, &[AgentDefinition])`: UPSERT into `agents` (`is_builtin = 1` always), preserving ID
  via `ON CONFLICT(slug) DO UPDATE`. Aliases sync **additive-only** — `INSERT OR IGNORE` each seed
  alias, never delete — so a user's own extra aliases (or edits) are never touched by a re-sync.
  Built-in agents whose slug disappears from a future `zzz.toml` are pruned only if they have zero
  mods attached (same rule as the old app's entity pruning).
- `sync_categories(tx, &Definitions)`: same UPSERT + prune-if-unused pattern as the old
  `sync_definitions`, targeting `categories`/`category_items` instead of `categories`/`entities`.
- `sync_definitions(conn, app_handle)`: reads the version from `settings.app_version`, compares to
  `app_handle.package_info().version`, skips entirely if unchanged (same gate as before); otherwise
  reads `definitions/zzz.toml` via the v2 resource resolver, runs both sync paths in one transaction,
  then writes the new version into `settings`.

Wired into `lib.rs`'s `setup()` right after `db::init_db()`, before `app.manage(...)`.

## Tauri commands (new)

- `list_agents() -> Vec<AgentWithAliases>`
- `get_agent(slug: String) -> AgentWithAliases`
- `create_agent(input: AgentInput) -> AgentWithAliases` — `is_builtin` forced to `0` server-side,
  regardless of what the frontend sends.
- `update_agent(slug: String, input: AgentInput) -> AgentWithAliases` — allowed for both built-in and
  user-added agents (per your answer above).
- `delete_agent(slug: String)` — returns an error if `is_builtin = 1`; enforced server-side, not just
  hidden in the UI, since this is a real data-integrity rule, not just a UX nicety.

`AgentInput` carries the full desired `aliases: Vec<String>` on every save; the backend diffs against
current rows (insert additions, remove what's missing) — **for user edits** this is a full replace,
unlike the additive-only seed-sync path above.

## Frontend: Add/Edit Agent UI

- `pages/agents/index.vue`: grid of agent cards (thumbnail, name, a small "Built-in" badge), "+ Add
  Agent" button.
- `pages/agents/[slug].vue`: becomes the edit form for an existing agent (was a placeholder in Phase 1).
- New `pages/agents/new.vue`: same form, blank, posts to `create_agent`.
- Form fields: name, slug (auto-derived from name on create via slugify, locked after creation —
  mods will reference agents by ID, not slug, but the slug is user-facing/URL-facing and changing it
  out from under an existing alias-matching setup seems more confusing than useful), description,
  alias chip-editor (add/remove pills), image picker, and a `details` sub-form.

## Flagged for your review — proposals, not yet confirmed

1. **`details` field shape**: currently just a raw string in the schema (mirrors the old app's
   JSON-blob-in-a-TEXT-column approach). For the edit UI I'm proposing **structured fields** —
   Rank (S/A dropdown), Attribute (Electric/Fire/Ice/Ether/Physical/Frost/AuricInk/HonedEdge — the
   real ZZZ attribute list per the seed data), Speciality (Attack/Stun/Anomaly/Support/Defense/
   Rupture), Type (multi-select: Slash/Strike/Pierce) — serialized to the same JSON-string shape
   under the hood so nothing else has to change. Reasonable default, but tell me if you'd rather
   just have a freeform textarea.
2. **User-added agent images — implemented simpler than proposed**: rather than copy-to-appdata +
   `convertFileSrc` + a new asset-protocol scope entry, went with a Rust command
   (`read_image_as_data_url`) that reads the picked file's bytes directly (arbitrary path — plain
   `std::fs::read` isn't subject to the fs-plugin's JS-side capability scope) and returns a
   `data:image/...;base64,...` URL. The frontend stores that string straight in `base_image` and
   renders it directly for user-added agents (built-ins still resolve `base_image` as a filename
   under `/images/entities/`). Zero new capability/scope surface area, at the cost of the DB row
   holding an embedded image — acceptable for small avatar-sized images on a solo desktop app.
3. **Alias-removal edge case**: since seed-sync is additive-only, if you remove one of my best-effort
   aliases via the edit UI, a future app update's re-sync won't re-delete it — but it also has no
   memory of "this one was explicitly removed," so if I ever re-add that same alias verbatim to a
   future `zzz.toml`, it'll reappear. Flagging as a known limitation rather than solving it now
   (would need a denylist table) — shout if this actually bothers you in practice.
4. **DB migration for the dev DB — resolved**: deleted the local dev DB once; the new schema
   (`category_items` + `mods.category_item_id`) was created fresh on the next launch. No auto-migration
   code added since there's no real user data yet.

## Unplanned fixes hit during implementation

- **51 characters, not 46** — REBUILD_NOTES/this plan both said "46" from an earlier miscount;
  `zzz.toml`'s `[characters]` table actually has 51 entries. All 51 got aliases and seeded correctly.
- **`vue-tsc` + TypeScript 7 incompatible**: `vue-tsc@3.3.8` resolves an internal `typescript/lib/tsc`
  path that TS 7's restructured package exports no longer expose — breaks `npm run build` entirely
  (it chains `vue-tsc -b && vite build`). Downgraded `typescript` to `^6.0.3` (latest pre-7 stable);
  vue-tsc's own peer range (`>=5.0.0`) is satisfied and everything type-checks clean. Worth revisiting
  once vue-tsc ships a TS7-compatible release.
- **`vite.config.ts` needed `@types/node`** for `process.env.TAURI_DEV_HOST` — added `@types/node` and
  `"node"` to `tsconfig.json`'s `types` array (single shared tsconfig, not split into app/node configs,
  consistent with Phase 1's "keep it simple" choice for this small scaffold).
- **Route ordering** (`pages/agents/[slug].vue` vs `pages/agents/new.vue`): the generated route array
  lists `:slug` before `new` (alphabetical file-scan order), which looked like `/agents/new` might
  match the dynamic route instead. Verified empirically with a standalone `vue-router` resolve test —
  its matcher scores static segments above dynamic ones regardless of array order, so `/agents/new`
  correctly resolves to the static route. No fix needed.

## Verification

1. Delete/recreate the dev DB, run `npm run tauri dev`, confirm all 51 agents + their aliases landed,
   plus the category/category_items rows. **Done** — 51 agents (`is_builtin=1`), 125 aliases (spot-checked
   Anby/Ellen/Soldier 0 - Anby for no collisions), 5 categories + 11 category_items matching `zzz.toml`
   exactly, `settings.app_version = "3.0.8"`.
2. Add a new agent through the UI, confirm it persists and shows `is_builtin = 0`. **Not done by
   Claude** — needs a manual click-through, no way to drive the native window from here.
3. Edit a built-in agent's aliases and image, confirm the change persists and a second app "restart"
   (re-running sync) doesn't clobber the edit. **Not done by Claude** — same reason as above.
4. Confirm attempting to delete a built-in agent is rejected (both UI-hidden and command-level).
   **Command-level enforcement verified by code review** (`delete_agent` checks `is_builtin` and
   returns `Err` before touching the row); **UI-hidden state not click-tested**.

`cargo check` and `vue-tsc -b` both run clean; the app compiles, launches, and the dev window is open
right now for you to click through items 2–4 yourself.

## Progress so far

- [x] `zzz.toml` extended with `aliases` per character (51, not 46 — see above)
- [x] Schema: `category_items` table + `mods.category_item_id` column
- [x] `src-tauri/src/db/seed.rs`: definitions structs + `sync_agents` + `sync_categories` + version gate
- [x] Wired into `lib.rs` setup
- [x] Tauri commands: list/get/create/update/delete agent (+ `read_image_as_data_url`)
- [x] Frontend: agents list, new-agent page, edit-agent form (name/description/aliases/image/details)
- [x] Automated verification (seed data, compile/type-check); manual click-through still pending (your side)

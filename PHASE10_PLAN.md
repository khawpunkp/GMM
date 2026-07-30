# GMM Rebuild — Phase 10: Search, Sort, Agent Filters, Back Button

## What this is

Not in the original roadmap — ad-hoc follow-up. User asked to add a search bar + sort to every
main listing page, an agent filter "like the old version", and a back button on sub-pages.
Researched the actual old app via git history (`37cc0c4^`, the last pre-rebuild commit) rather than
guessing — everything below is grounded in real old code, not invented.

## What the old app actually did (verified, not guessed)

Lived in exactly two old files: `HomePage.jsx` (grid of entities within one category — the old
app's "characters" category is what our `agents/index.vue` is) and `EntityPage.jsx` (grid of mods
for one entity — what our `agents/[slug].vue` is).

- **Search**: client-side substring filter over the already-loaded list, no backend call.
  `HomePage.jsx` matched entity **name** only. `EntityPage.jsx` matched mod **name + author**.
- **Sort**: a `<select>` in the page header, persisted to `localStorage` per page, default
  `name-asc`. `HomePage.jsx`: Name A-Z/Z-A, Total Mods High/Low, Enabled Mods High/Low ("Other"
  entities always pinned first regardless of sort). `EntityPage.jsx`: Name A-Z/Z-A, Date Added
  Newest/Oldest (by DB `id`), Status Enabled/Disabled-first.
- **Agent filter**: on `HomePage.jsx` only, and only for the characters category — three
  single-select chip rows (Rank / Attribute / Speciality), read from each agent's `details` JSON,
  AND-combined, click-again-to-clear. (A separate per-mod "type" chip filter also existed on
  `EntityPage.jsx` — not part of what was asked, flagging but not building it.)
- **Back button**: `EntityPage.jsx` only — an arrow icon next to the title, `router.back()` with a
  fallback to the characters grid if there's no history depth. `HomePage.jsx` had none (it's
  top-level).

## Mapping onto the rebuild's (different) page structure

Our structure isn't 1:1 with the old app's — Phase 8 deliberately made the 5 non-Agent categories
flat (mods directly, no entity sub-grid), so there's no exact old-app equivalent for
`categories/[slug].vue`. Mapping used:

- `pages/agents/index.vue` = old `HomePage.jsx` for characters → search (agent name) + sort (all 5
  old options) + Rank/Attribute/Speciality chip filters. No back button (top-level).
- `pages/agents/[slug].vue` = old `EntityPage.jsx` → search (mod name + author) + sort (all 3 old
  options) + back button (`router.back()`, falls back to `/agents`).
- `pages/categories/[slug].vue` = no old equivalent (old app always had an entity sub-grid step
  here). Since it shows mods directly, it inherits `EntityPage.jsx`'s search/sort shape, not
  `HomePage.jsx`'s — same 3 sort options as the agent mod page. No back button (top-level, reached
  directly from the sidebar, matching why old `HomePage.jsx` had none).

## Judgment calls (flagging, not blocking on)

1. **Groups + search/sort**: didn't exist in the old app. Search will also match against group
   names (so searching hides non-matching groups too); sort only reorders the ungrouped mod list —
   groups keep rendering first, unsorted, same as today.
2. **Chip filter values**: derived dynamically from whatever `rank`/`attribute`/`speciality`
   values actually appear among loaded agents, not hardcoded to the old app's fixed list — future
   user-added agents with new values show up automatically instead of being unfilterable.
3. **"Total Mods" / "Enabled Mods" sort** needs per-agent mod counts, which `list_agents` doesn't
   return. Fetches all mods once (`list_mods` with every filter `null`) and counts client-side in
   `agents/index.vue`, without touching the shared `modsStore.mods` state other pages rely on.
4. **Sort persistence**: kept the old app's `localStorage` behavior — one key per page context
   (`sort_agents`, `sort_agent_<slug>`, `sort_category_<slug>`).

## Progress so far

- [x] `pages/agents/index.vue` — search (name) + sort (6 options) + rank/attribute/speciality
      chips (dynamically derived, AND-combined, click-again-to-clear); mod counts for the
      Total/Enabled sort options come from one `list_mods` call with every filter `null`, kept
      local to this page rather than mutating `modsStore.mods`
- [x] `pages/agents/[slug].vue` — search (name + author) + sort (6 options) + back button
      (`router.back()`, falls back to `/agents` when there's no history depth); search also
      filters group names, sort only reorders the ungrouped list
- [x] `pages/categories/[slug].vue` — same search/sort shape as the agent mod page, no back button
- [x] `vue-tsc -b` clean
- [x] Sort choice persisted to `localStorage` per page context, matching the old app

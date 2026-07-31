# GMM Rebuild — Phase 5: Mod Grouping (Virtual)

## Context

Phases 1-4 done, committed, pushed. This is Phase 5 per `REBUILD_NOTES.md`: "Mod grouping (virtual):
group table/relation, merge/split UI, group-level toggle." Confirmed scope (REBUILD_NOTES point 7):
**virtual grouping only** — folders stay separate on disk, the DB gets a group/parent relation, the UI
treats grouped mods as one toggle unit. Checked the legacy code for prior art first: there is none —
`grep` for "group" in `_legacy/main.rs.reference` returns nothing. This is new functionality being
designed from scratch, not a port, unlike every previous phase.

Two real design decisions needed a call before the schema/commands could be written correctly (both
resolved before writing this doc):

1. **A mod belongs to at most one group** — adding `UNIQUE(mod_id)` to `mod_group_members` (schema
   change below). Allowing a mod in multiple groups simultaneously has no obvious UI answer (which
   combined card does it show up in?), so this is enforced at the DB level, not just convention.
2. **Group toggle semantics when members are in a mixed state**: a group displays as "enabled" only
   when _every_ member is enabled. Clicking the toggle: if all are currently enabled, disable all;
   otherwise (mixed or all-off), enable all. Standard "select-all checkbox" behavior — always resolves
   to a clean all-on or all-off state in one click, never ambiguous.

## Schema change

```sql
CREATE TABLE mod_group_members (
    group_id INTEGER NOT NULL,
    mod_id   INTEGER NOT NULL UNIQUE,
    PRIMARY KEY (group_id, mod_id),
    FOREIGN KEY (group_id) REFERENCES mod_groups(id) ON DELETE CASCADE,
    FOREIGN KEY (mod_id)   REFERENCES mods(id)        ON DELETE CASCADE
);
```

Only change from Phase 1's original: `mod_id` gains `UNIQUE`. Same "edit schema.rs directly, delete
your dev DB once" approach as Phase 2's `category_items` addition — no real user data to migrate yet.

## Backend

`src-tauri/src/mod_groups.rs` (business logic, same split-from-commands pattern as `mods.rs`/
`presets.rs`):

- `create_group(conn, name, mod_ids)` — validates every mod_id exists and is currently ungrouped
  (rejects if any is already in a group — no auto-merge/steal), inserts the group + members.
- `list_groups(conn, base_mods_path)` — each group's member mods + a computed `is_enabled` (all
  members enabled) using the same `is_mod_enabled` check everything else already uses.
- `add_member(conn, group_id, mod_id)` — mod must be currently ungrouped.
- `remove_member(conn, group_id, mod_id)` — "split" one mod off. If the group would end up with ≤1
  member, auto-delete the group instead (a "group" of one mod isn't meaningfully a group).
- `rename_group`, `delete_group` (disbands only — cascade removes membership rows, mods themselves
  and their on-disk folders are completely untouched, per "virtual grouping").
- `toggle_group(base_mods_path, conn, group_id)` — the semantics from point 2 above, reusing
  `mods::toggle_mod` per out-of-sync member (same rename mechanism, nothing new there).

New commands (`commands/mod_groups.rs`): `list_mod_groups`, `create_mod_group`, `add_mod_to_group`,
`remove_mod_from_group`, `rename_mod_group`, `delete_mod_group`, `toggle_mod_group`.

`mods::list_mods`/`ModWithState` gains a `group_id: Option<i64>` (a `LEFT JOIN` against
`mod_group_members`) so the frontend knows which mods are already grouped — needed both to render
grouped mods as a single combined card and to know which mods are selectable for a _new_ group
(only ungrouped ones).

## Frontend

- `GroupCard.vue` (new, parallel to `ModCard.vue`): group name, one toggle switch (the group-level
  toggle), expand/collapse to show member mods (each with its own "Remove from group" + open-folder
  action), rename, and "Ungroup" (disband — `delete_mod_group`, not a mod delete).
- Agent detail page's Mods section gains a **selection mode**: a checkbox appears on each _ungrouped_
  ModCard; once 2+ are checked, a "Group Selected (N)" button appears, prompts for a name, calls
  `create_mod_group`. Rendering logic partitions `modsStore.mods` by `groupId` — one `GroupCard` per
  distinct group, one `ModCard` per still-ungrouped mod.
- New `stores/modGroups.ts`: fetchAll/create/addMember/removeMember/rename/delete/toggle, same shape
  as the existing mods/presets stores.

## Scoped narrower than the data model actually allows — flagging up front

The backend places no restriction on a group spanning mods that belong to _different_ agents or
categories — `create_mod_group` just takes a list of mod IDs. But the only place mod cards currently
render is the single-agent Mods section (`pages/agents/[slug].vue`), so the selection-mode UI this
phase builds can only group mods you can see together on one agent's page. Cross-agent grouping (e.g.
a "full outfit" spanning a body mod filed under one agent and an accessory mod filed under another,
or including an uncategorized mod) isn't reachable through this phase's UI even though the data model
already supports it. A global mods-browser page would be needed to close that gap — not building one
this phase; REBUILD_NOTES doesn't ask for it here and it's a bigger scope item on its own.

## Unplanned addition: a real migration, not another "delete your dev DB"

Every prior phase's schema change (Phase 2's `category_items`, Phase 3 needed none, Phase 4 needed
none) was handled by just telling you to delete the local dev DB — fine when it held almost no real
data. That convention doesn't hold anymore: by this phase your DB has 51 real agents, 74 real scanned
mods, and your corrected `mods_folder_path`. Wiping it to pick up one `UNIQUE` constraint would have
cost you a full re-scan for no reason. Added `db::migrate_mod_group_members_unique()` instead — a
one-time, flag-gated migration in `db/mod.rs` that runs _before_ `schema::SCHEMA`: drops
`mod_group_members` if it exists in its pre-constraint shape (always safe, since the mod-grouping
feature is brand new — no prior release could have created a real row in it), lets `schema::SCHEMA`
recreate it correctly, then marks a `settings` flag so this never runs again. Verified directly
against your live DB (not a synthetic test): the constraint is now in place, the flag is set, and all
other real data — agents, mods, settings — came through untouched.

## Verification plan

**Automated** (`cargo test`, all 11 pass across the whole crate now):

- `create_and_toggle_group_with_mixed_state` — creates a group of 2 synthetic mods, disables one to
  produce a mixed state, confirms the group reads as not-enabled, confirms toggling a mixed group
  turns everything ON (not off), confirms toggling an all-on group turns everything OFF.
- `create_group_rejects_already_grouped_mod` — confirms the one-group-per-mod rule is enforced.
- `remove_member_auto_deletes_group_when_down_to_one` — removes members one at a time from a 3-mod
  group, confirms auto-delete once down to 1, confirms the last member's own membership row is
  cleaned up too (not left dangling).

`cargo check` and `vue-tsc -b` both clean. Migration verified live against your real DB as described
above, not just a synthetic fixture.

**Manual, flagged for you**: actually selecting mods and grouping them through the UI,
expanding/collapsing a `GroupCard`, removing a member, disbanding a group.

## Progress so far

- [x] Cardinality and toggle-semantics decisions confirmed
- [x] Schema: `UNIQUE(mod_id)` on `mod_group_members` + live migration (verified against real data)
- [x] `mod_groups.rs` + commands + `ModWithState.groupId`
- [x] Frontend: `GroupCard.vue`, selection-mode UI, `modGroups` store
- [x] Verification checklist above — automated tests + live-DB migration check done; UI click-through
      still needs your hands-on pass

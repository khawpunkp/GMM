# GMM Rebuild — Phase 6: Self-Update

## Context

Phases 1-5 done, committed, pushed. This is Phase 6 per `REBUILD_NOTES.md`: "Self-update: Tauri v2
updater plugin + release endpoint + UI." Unlike every other phase, this one started without a
pre-written plan doc — the design emerged through conversation, so this file is written after the
fact to keep the same record as every other phase, not before.

## The real blocker found before any code: the updater config wasn't yours

Phase 1 carried over the old app's `plugins.updater.pubkey`/`endpoints` unchanged ("same signing key,
same update feed" — its own words, in hindsight the wrong call). Checked before writing anything this
phase: both belonged to **Eidenz**, the original pre-rebuild author.

- The `pubkey` is a minisign public key — only whoever holds the matching *private* key can sign
  updates that validate against it. There was no way to ever publish a release this rebuilt app would
  accept.
- The `endpoints` URL pointed at Eidenz's own live GitHub Pages feed for their original (pre-rebuild,
  React + Tauri v1) app — not just wrong, but pointed this app at a stranger's release infrastructure.

Self-update could not have worked at all until this was replaced with infrastructure the user
actually controls. Resolved this turn:

1. Generated a fresh minisign keypair via `npx tauri signer generate`. Private key +
   password live outside the repo (`~/.tauri/gmm-updater.key` / `~/.tauri/gmm-updater.password.txt`
   on this machine) — never printed into any tool output, to keep them out of this transcript.
2. `tauri.conf.json`'s `pubkey` swapped to the new public key (safe to commit — it's public by
   design); `endpoints` swapped to `https://github.com/khawpunkp/GMM/releases/latest/download/latest.json`.
3. `.github/workflows/release.yml` — triggers on a `v*.*.*` tag push, builds on `windows-latest`,
   signs via `tauri-apps/tauri-action@v1` using two new GitHub Actions secrets
   (`TAURI_SIGNING_PRIVATE_KEY`/`_PASSWORD`), publishes a **draft** release. Draft is deliberate —
   the GitHub "latest release" endpoint only resolves published, non-prerelease releases, so nothing
   ships to existing installs until the user reviews and publishes it by hand.
4. `RELEASE_CHECKLIST.md` — one-time secrets setup + the per-release steps (bump 3 version files in
   sync, tag, push, review the draft, publish).
5. Secrets added — `gh` stayed authenticated as a different account (`krittayos-p`, read-only on this
   repo) than the one that owns `khawpunkp/GMM`, so this went through GitHub's web UI directly rather
   than `gh secret set`. Not independently verified from here (the same read-only auth blocks
   `gh secret list` too, 403) — taking the user's confirmation at face value.

## App-side implementation

- `@tauri-apps/plugin-updater` + `@tauri-apps/plugin-process` (npm) and their matching Rust crates
  (`tauri-plugin-process` newly added; `tauri-plugin-updater` was already a dependency since Phase 1,
  registered but never actually used until now). `process:default` added to capabilities (grants
  `allow-restart`, used by `relaunch()`).
- `stores/updater.ts`: wraps the plugin's own `check()`/`Update` object directly rather than
  redefining its shape as a custom DTO — unlike the Rust-backend features in this app, the updater
  plugin's JS API already returns a fully-typed object, so there's nothing to wrap. Tracks
  checking/downloading state, download progress (from the plugin's `Started`/`Progress` events),
  and closes the previous `Update` resource before replacing it (`Update` is a Tauri `Resource`, needs
  explicit cleanup) — `check()` never rejects, catches its own errors onto `errorMessage`.
- `UpdateModal.vue`: version + release notes, download progress bar, Download & Install → Restart Now.
- Settings gained an "Updates" section: current version (`getVersion()` from `@tauri-apps/api/app`),
  a manual Check for Updates button, inline "you're up to date" / error status.
- Sidebar: a small accent-colored dot on the Settings nav item when an update is available — no
  unprompted modal on startup, matching the same "subtle, discoverable, not intrusive" reasoning used
  for the Phase 4 launcher's placement.
- `App.vue`: a silent `updaterStore.check()` on mount populates the Sidebar badge without popping
  anything open.

## Verification

- `cargo check` + `cargo test` (still 11 passing — this phase added no new Rust business logic worth
  unit-testing; it's plugin registration + capability config + frontend) + `vue-tsc -b`, all clean.
- Confirmed the real endpoint returns a clean `404` right now (no release published yet) via `curl`,
  and that the store's `check()` absorbs that as an error rather than crashing — the dev app stayed
  up through the rebuild.
- **Not tested**: an actual end-to-end release (tag push → Actions run → draft → publish → app detects
  it → downloads → installs → relaunches). Can't be, until a first real version is tagged and
  published. This is the natural next real-world test once there's something worth shipping.

## Progress so far

- [x] Own signing keypair generated, `tauri.conf.json` updated
- [x] GitHub Actions release workflow + `RELEASE_CHECKLIST.md`
- [x] Secrets added to the repo (via GitHub's web UI, not `gh` — see note above)
- [x] In-app check/download/install/relaunch UI
- [ ] A real tagged release, to confirm the whole chain end-to-end (blocked on the user actually
      wanting to ship a version — not something to do speculatively)

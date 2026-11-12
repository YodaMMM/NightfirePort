# level-14-kiko-corridor-2 — report

Branch `s/level-14-kiko-corridor-2` = `origin/s/level-14-kiko-corridor` f6dd48825 with
`origin/master` merged on top (merge commit 2fb72bf2f, clean — master touched
walkreal_07000011/13/15 tests and level sound start; nothing conflicts with 07000014).
WORKLOG entry `level-14-kiko-corridor-2` pushed as 8a5f6a814.

**Status: blocked — the task's own work (the three levers, the walk runs, the suite) did not
start. The sandbox disk filled before the test build went green.** Nothing about the d1 walk
changed: `walk_the_infiltration_to_the_delta_exit_real_damage` keeps its previous ignore
message verbatim (the tick-43690 death at (−58.92, 0.73, 37.67) to the west-hunting guards
#19/#23/#24 and the three ordered levers). No `crates/*/src/` change, no test change landed.

## What was done

1. **Setup.** Repo present from the previous session; token URL reset, fetched; the task
   branch created from `origin/s/level-14-kiko-corridor` and `origin/master` merged
   immediately. `work/` (slim) and `Nightfire-onfoot.iso` already extracted;
   `NF_ISO` re-exported per shell. `../nf-queue` refreshed to origin/queue 83cdf3e26.
2. **The merge's Cargo.lock needed a fresh fetch** — the first `cargo test --offline` died
   with `failed to download ab_glyph v0.2.32`; plain `cargo fetch` (network) fixed it. The
   next session should expect this if master moved again: run `cargo fetch` once WITHOUT
   `--offline` before any offline build.
3. **Disk surgery under a full 9.9 GB rootfs** (this is the useful part for the next session):
   - `target/release/deps` held 6.0 GB from the earlier same-sandbox sessions. A dedup pass
     (keep the newest artifact per crate-name+extension base, delete older hashes) freed
     ~0.2 GB; deleting `~/.cache/ms-playwright` + `~/.cache/puppeteer` (browser binaries of
     the web tooling, unused by Rust) freed ~1.2 GB; deleting `~/.cargo/registry/cache`
     (only the .crate archives — `registry/src` stays, offline builds still work) and
     `/tmp` junk a bit more.
   - **Stripping the already-built test binaries saved 1.34 GB** (`strip target/release/deps/<test-bin>-*`
     for extension-less executable files — safe: cargo's freshness does not hash the binary,
     the suite runs it the same; only panic backtraces lose symbol names).
   - Even so, every `cargo test --no-run` minute re-filled the disk (each finished test
     binary lands unstripped ~10-15 MB, and superseded copies are kept by cargo forever).
     The build never completed: last state was `nf-view` still compiling with 7 MB free.
   - **Recipe for the next session (do this first):** dedup + strip as above, then build with
     `cargo test --offline --release --no-run --tests -p nf-view` (`--tests` skips the 385 MB
     `release/examples` dir, which plain `cargo test` rebuilds), strip the fresh binaries
     whenever `df` drops under ~300 MB, and only then start `tools/test/suite.py`. Budget:
     the four-crate test-binary set plus examples does not fit comfortably on this rootfs
     alongside the ISO, `work/` and the toolchains without the strip pass.

## What is left (the task itself, unchanged from the ledger)

The ignore message's three levers on `walk_the_infiltration_to_the_delta_exit_real_damage`,
in order, each judged over real runs (the sim is deterministic — two runs confirm
byte-identity), all gated `difficulty == 1 && !w.tops_up()` exactly like the existing pocket
levers so the plain walk stays byte-identical and the d2/d3 sisters are untouched:

1. **Lever 1 — a second `clear_hostiles` pass after the corridor waypoint.** Insertion point:
   `crates/nf-view/tests/walkreal_07000014_live.rs`, immediately after the strict waypoint leg
   `w.walk_to(Wp::new(-49.2, 0.9, 38.4).r(1.0), "the Kiko corridor east");` (the last line of
   the `!tops_up` else block) and before `w.follow_nav_route([-33.1, 0.9, 26.8], "east to the
   escape touch #456");`. Call `clear_hostiles(&mut w, 14.0, 3600);` — the death spot is
   ~9.8 u from the waypoint, inside the 14 u radius, and `clear_hostiles` re-scans after each
   kill so hunters that walk into the radius during the pass die too.
2. **Lever 2 — the fight-quiet window over `follow_nav_route`'s first legs.** Save
   `w.dmg.return_fire`, set it false around the `follow_nav_route` call, restore it at the
   escape-touch branch (before the `0x4a` wait loop). No shared-file change: this is the same
   knob the pocket legs already use, set from the test body.
3. **Lever 3 — the resupply/loot pass on the corridor floor before re-engaging.** The shared
   `w.resupply(label)` (master rules: 45 u fetch, 1.5 floor band, 1.8 arrive) called between
   the quiet window's end and the return_fire restore; if the touch fires mid-resupply the
   `0x4a` loop reads the channel and breaks immediately, so placement is safe.

Then: un-ignore when green (or push the ledger + the next lever in the ignore message if
still red), run `walkreal_07000014_live` whole and `interactables_07000014_live` whole, run
`tools/test/suite.py` on the final tree and report the total line.

The 00 Agent walk's ledger is unchanged (the "1.72 landing" hover — not this task).

## What ran

Nothing new — no test binary finished building inside the window, so no run of any test is
reported by this session. The branch carries only the merge + WORKLOG + this report. The
previous session's numbers stand: walkreal_07000014_live whole 8 passed / 2 ignored,
interactables_07000014_live whole 10 passed (on the pre-merge tree).

## Live checks for the main agent

No new checks from this session (nothing was measured). The two checks in
`docs/reports/level-14-kiko-corridor.md` ("Live checks for the main agent" 1-2: the fresh-boot
`switch_channels[60]` @ 0x26fccc read at Operative and Agent, and the machine-room pocket
steer) remain open and unchanged.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage panel is exposed to the agent).

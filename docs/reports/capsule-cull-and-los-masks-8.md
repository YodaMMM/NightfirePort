# capsule-cull-and-los-masks-8 — the head-on cull and per-caller LOS masks are done; the remaining named group (the 09 body-armour take) already passes; the dead gates deleted

## What's done

1. **The task's named remaining item — interact_07000009_live's body-armour take — is green
   on this tree, and had been fixed before this window started.** The task brief (written
   against the ac8c16bb tip) says "the body-armour take never fires (Bond on the right
   floor)". On the current branch tip (7f639227 + the master merge, re-verified twice this
   window, before and after this window's edits) the whole binary passes:
   `walking_over_the_body_armour_takes_it` and `standing_short_leaves_the_armour` included —
   **16 passed, 0 failed** both runs. The fix that resolved it is `ebdba184
   interact_07000009_live: settle_near stands at the game's height (floor + 1.1)`, which is
   an ancestor of ac8c16bb (it does not appear in `git log ac8c16bb..origin/s/capsule-cull-
   and-los-masks-7`): `settle_near` casts 0.5 above the spot down 30 units, then stands Bond
   at floor + 1.1 [M Player_StandAtNewPosition 0x190d88:0x190ee0-0x190f18], so the take gate
   (armour world-sphere radius 0.4767 [D probe] + 1.1 = 1.577 [M 0x18e148:203-210]) is
   actually reachable. No code change was needed; nothing was weakened.

2. **The masks-7 leftovers are closed.** (a) The dead `LOS_MASKS_ON` / `HEAD_ON_CULL_ON`
   consts (`pub const false`, read by nothing — verified by grep across the repo: only two
   comments mentioned them) are deleted; the stale comment blocks that still said "the
   per-caller masks are not applied yet [S]" and "Still OFF" are replaced by one block that
   states both features are unconditional with their cites [M Collide_Intersect
   0x1ebaa8:300-319, Collide_Pick 0x1eb320:140-152; M Intersect_CylGeom 0x109da0:302-322]
   and keeps the level-0a-office-hatch-9 arms-orientation finding (below-spine planes clamp
   f=1, above-spine clamp f=0; the first-cut arms had the f halves swapped). The two
   referencing comments (drone.rs GT-track LOS block, mp_net_slots_post_move_live evidence
   block) are re-pointed. Comment-only delta — no behaviour change, and the digest pins
   below confirm it. (b) The two `#[ignore]`d world.rs tests from the masks-7 report are
   the assertion-free `stairwell_bullet_census_07000011` / `stairwell_walls4_census_07000011`
   censuses (ignore reason: "run with --ignored --nocapture") — intentionally ignored, not
   the stale cull ignores the report remembered; nothing to un-ignore. (c) The
   band-settle-3 radial arm (b55121af) IS in the lineage now (it came in with master
   fcf6f334, verified `git merge-base --is-ancestor b55121af HEAD`), so the walk families
   were re-run: all green (below).

3. **No new decompile conflicts found.** The task's "fix each from the game's rule [M]"
   loop over the masks-7 diagnosed/pending groups reduces to the above: the named group
   passes, the other three were bookkeeping (dead consts, stale ignores, the band-settle-3
   re-run). Nothing contradicted the decompile this window, so no test was re-derived and
   none weakened.

## How it was verified (what ran, on the final tree = 7f639227 + this window's comment commit)

- `interact_07000009_live` (nf-view): **16 passed, 0 failed** — run twice (before the
  edits and again after the rebuild).
- `ai_live` (nf-game): **37 passed, 0 failed** (includes the seed-8
  `guard_takes_cover_like_the_game` re-pin from masks-7).
- Walk families with band-settle-3 in the lineage: `walkreal_07000001_live` 3/0,
  `walkreal_07000007_live` 2/0, `walkreal_07000008_live` 4/0, `walkreal_07000009_live`
  3/0, `walkreal_07000011_live` 1/0, `walkreal_07000014_live` 2 passed + 1 ignored (the
  documented level-14 leg-2 blocker, pre-existing), `probe_07_route` 1/0.
- `mp_net_slots_post_move_live` (nf-view): **2 passed** — the 0xbc5e_2155_a378_6e9c cull
  digest pin holds through the comment-only edits.
- `nf_game` lib test binary (world.rs + every nf-game unit test): **581 passed, 0 failed,
  2 ignored** (the two assertion-free censuses).
- **The fast-runner sweep** (`tools/test/suite.py`-equivalent; the nohup/setsid background
  runner dies between tool calls in this sandbox too — 8th sandbox measured — so the
  per-binary resume runner ran in foreground chunks, two in parallel on the 2 cores,
  `--test-threads=1`, 300 s per-binary cap): **278 of 874 test binaries run, 275 OK** at
  the stop-work mark, covering both ends of the alphabetical list (camera_*/cutscene_*/*gunimp*
  /abseil_*/ai_* families from the front, x*/w*/v* … from the back). Three non-OK:
  - `disc_golden` FAIL — the slim-ISO expected failure (task rule: only the 3 disc_golden
    tests may fail on the slim ISO).
  - `fix_0c_pylon_live` FAIL — environment, not code: wgpu `adapter: NotFound` (the sandbox
    exposes no Vulkan/GL adapter — the no-GPU class; visual work needs the main agent's
    render per the merge lessons). Not touched by this branch's delta (comment-only).
  - `campaign_flow_live` TIMEOUT ×3 — the resume runner's 300 s per-binary cap; suite.py's
    own default limit is 600 s. A runner artifact to re-run with `--limit 600` (left below).
  - The remaining ~596 binaries did not fit the window (setup from a cold sandbox: clone +
    downloads + rustup + a ~20 min cold release build consumed the first hour).

## What's left

1. Finish the sweep: ~596 binaries unrun (the runner state is in `suite-progress.txt`
   format, `OK/FAIL/TIMEOUT <name>`; the exe list rebuild command is in
   `tools/test/suite.py --no-run` + the json query, or reuse
   `tools/test/run_suite_resume.sh` with this sandbox's alsa prefix). Re-run
   `campaign_flow_live` with the 600 s cap.
2. `fix_0c_pylon_live` needs a GPU sandbox (or the main agent's render) to distinguish
   environment from code — the panic is at the wgpu adapter open, before any test logic.
3. Nothing else named by the task remains: the cull and the masks are unconditional and
   pinned (the 0xbc5e digest, the 581 nf-game unit tests, the walk/ai/interact families).

## Live checks for the main agent

None new this window — the window changed comments only; the cull/masks behaviour was
live-checked in the -6 window and the digest pins hold.

## Usage

not visible to me — this environment exposes no token, cost or context-window figures
for this session (no usage banner, no /cost, no token readout anywhere in the interface
this worker sees).

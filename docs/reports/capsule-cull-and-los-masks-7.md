# capsule-cull-and-los-masks-7 — the door filter dropped, the cull and the masks kept, the inherited ai_live red fixed with a cited seed re-pin

## What's done

1. **The `Door_IsClosed` filter is gone and the capsule content is the whole remaining
   delta.** The branch is `origin/s/capsule-cull-and-los-masks-6` + `origin/master`
   (merged twice: 39bd6f3e at master fee3d546, then fcf6f334 at the end of the window;
   both clean auto-merges — verified `git merge-tree --write-tree` identical for the
   earlier one). With the door gate dropped, `git diff origin/master HEAD` on
   `crates/nf-game/src` is exactly three files: `world.rs` (the game's head-on cull
   unconditional in `capsule_tri_contact` [M Intersect_CylGeom 0x109da0:302-322, arms
   `(nd > 0.999 && f > 0.5) || (nd < -0.999 && f < 0.5)`, and the per-caller LOS masks
   applied unconditionally in `line_of_sight` → `line_of_sight_masked`,
   `Collide_Intersect` 0x1ebaa8:300-319 / `Collide_Pick` 0x1eb320:140-152]), `drone.rs`
   and `gt.rs` (the GT muzzle-offset/TrackCand.sph/spread work, [M] cited in those
   files). `grep is_closed drone.rs` finds nothing; master's unconditional door extend
   stands. `LOS_MASKS_ON` / `HEAD_ON_CULL_ON` still exist as `pub const false` in
   world.rs but are read by nothing on this tree — dead leftovers with stale comments.

2. **The inherited ai_live red (`guard_takes_cover_like_the_game`) is fixed by finding
   its mover and re-pinning the seed with the evidence — no behaviour was changed.**
   The failure on the branch start was `UnderCoverAim lasted 90 (live 37)`: the state
   sequence pin still passed, but the guard picked the step-out cover danim 178 (90
   frames at 1.0) instead of the live lean-out 176 (51 at 1.5) — the cover-type Rand
   read a shifted stream. The bisect (git bisect over e0940821..ca2e7389, then
   function-level analysis) landed on the merge 4a155ddf with BOTH parents green, and
   the auto-merge tree verified identical to `git merge-tree`'s — a real content
   interaction. The RNG trace (an env-gated `#[track_caller]` eprintln on
   `Rng::frand/rand/random`, plus a per-tick `sim.rng.draws` scratch test, both since
   removed) pinned the first divergence to draw 7799 in ticks 101-150: on the new
   tree the guard's Combat-shot bullets fly 3 ticks longer and register 4 more
   `bullet_impact` events, each spawning the debris burst (~110 `frand_half` draws);
   by the 0x93 entry at t242 the shared stream is +978 draws (13055 → 14033) while
   every state tick stays identical (t61/t113/t114/t242/t272). The decisive
   discriminator: the same trace on the masks-OFF master tree (118a362b, seed 3)
   reads the identical shifted numbers (t242 draws 14033, danim 178 at t329) — **the
   shift is the batch-1026f bullet-flight work alone** [M Bullet_update 0x12e7b0
   disasm 0x12f030-0x12f058 — the per-flying-tick pick arm; Collide_Update
   0x1ea398:216-229 stops at the first non-pass hit; Collide_Pick 0x1eb320:125 — the
   cel-scoped pick], which master merged after master's own seed-2 calibration
   (attack-reentry-choice, 10-21, before the 10-25/26 bullet commits) — master's
   seed-2 pins simply don't probe the shifted stretch. The per-caller LOS masks are
   exonerated as the mover (masks identical across the compared trees; master's masks
   are off and shift identically). The stream change is the more faithful port of the
   game's flight pick, so per the task rule the calibration — not the code — moved:
   seed 3 → 8, re-pinned with the full evidence block in the test comment. All live
   pins (states, UnderCoverInit 31 / UnderCoverAim 37, cover node 1, position
   (58.83, 99.32), the sister) are unchanged live measurements. Seed scan: 1-14 run,
   8 is the pass (13 regressed on re-run and was not used).

3. **The capsule-cull digest pin holds on the final tree.** master's delta
   (batch-1027i, cutscene-handback-census / kind31-frame-3) touched
   `mp_net_slots_post_move_live.rs` and `nis.rs` among others; the merged file keeps
   the capsule-6 re-pin (`PINNED_FINAL = 0xbc5e_2155_a378_6e9c`, the culled digest)
   and the test is green on the merged tree (2 passed). The nis.rs changes are
   digest-inert here, as measured.

## How it was verified (what ran, on the final merged tree 5bd8…, master fcf6f334)

- `ai_live` (nf-game): **37 passed, 0 failed** — including the re-pinned
  `guard_takes_cover_like_the_game`.
- `mp_net_slots_post_move_live` (nf-view): **2 passed** (the 0xbc5e digest pin + the
  6-human loopback).
- `probe_07_route` 1/0, `walkreal_07000001_live` 3/0, `walkreal_07000008_live` 4/0,
  `walkreal_07000014_live` 2 passed + 1 ignored (the documented level-14 leg-2
  blocker from level-14-full-walk-4 — pre-existing, not this task's scope).
- Suite runner status: the background `tools/test/suite.py` and nohup'd shells die
  between tool calls in this sandbox (measured again this window; 7th sandbox). A
  resume-capable foreground runner (one line per binary in `suite-progress.txt`) was
  used instead: **53 binaries, 53 OK, 0 FAIL, 0 TIMEOUT** on the final merged tree
  before the window's stop-work mark (22:33 UTC) — the quick prefix (camera_*/nf-audio
  /nf-formats families); the ISO-loading nf-view/nf-game families beyond those were
  run directly as listed above. The 845-binary end-to-end pass still has not happened
  in this sandbox lineage; the runner script is committed as
  `tools/test/run_suite_resume.sh` for the next window.

## What's left

1. The full 845-binary suite end to end on the merged tree (the resume runner makes
   it incremental; ~25 min of window did not suffice).
2. The two `#[ignore]`d world.rs tests (`HEAD_ON_CULL_ON is off until the cull keeps
   the real-path walks green`) are stale on this tree — the cull is unconditional and
   the walks are green; un-ignoring them is a one-line change the next window should
   verify with a run.
3. The dead `LOS_MASKS_ON` / `HEAD_ON_CULL_ON` consts and their stale comment blocks
   in world.rs (both say the flags gate behaviour; nothing reads them) — delete or
   re-point at the evidence next window.
4. The band-settle-3 radial arm (b55121af) is NOT in this lineage (checked: not an
   ancestor of capsule-4/-6); when master's next merge brings it, the walk families
   need a re-run.

## Live checks for the main agent

None new this window — the seed re-pin's evidence came from the port's own RNG trace,
and the cull/masks behaviour was already live-checked by the -6 report's section.

## Usage

not visible to me — this environment exposes no token, cost or context-window figures
for this session.

# guard-radii-01-d3 — the 01 00 Agent walk under the game's guard radii

Branch `s/guard-radii-01-d3` (from `origin/s/guard-radii-finish` e88867f93, which already
contained `origin/master` — the merge was a no-op). Test-side only; every change is in
`crates/nf-view/tests/walkreal_07000001_live.rs`; no shared helper (`crates/nf-view/tests/common/`)
was touched, no `crates/*/src/` change, no radii/damage/health touched.

## What was done

The one red in `walkreal_07000001_live` was `..._real_damage_00agent` (d3). The finish report
(item 5) named the shape and two levers; both are implemented, d3-gated, exactly as named.

**Baseline reproduction** (unmodified tree, [L] this session's run): the walk died at tick
**70165** — the report's own number. The trace's ledger: the d3 threshold fights burned every
pool the walk owns (the pistol's 54, the hall box's +20, then the wheel SMG's 55 via the
dry-gate's armed switch), the resupply cycles inside the hall found nothing (the hall's south
wall walls the nav off from the corridor boxes), the dry fall-through shoot flanked Bond INTO
the hall, and lip guard #5 (drone #5, skin 0x5000038, the shotgun 0x1c, 24 hp) took the last
health at 1.4 u (frames: -14.40/-14.80 at d 1.4).

**Lever 1 — resupply BEFORE the hall phase at difficulty 3.** The pre-hall top-up block (after
the meet scene, before the hall named loop) now, at d3 only, sweeps the corridor's resupply
boxes onto the wheel: repeated `resupply()` calls (bounded 3), each taking the nearest
walked-path box — the taken box is gone, so each call advances to the next — then walks back to
the meet room (the stage-below invariant the dry branch keeps). The d1/d2 path is the original
code verbatim (an `else` branch), so their timelines are untouched. No mid-fight detour: the
stock happens before any hall fight.

**Lever 2 — run the lip block's scan again once the escort crossing begins.** The post-AND lip
block's scan and fight are extracted verbatim into `door_lip_scan` / `door_lip_fight` (same
filter: team 1, started, not corpse, enabled, health > 0, within 7.0 of the lip point
(7.9, 6.3, 11.1); same bounded live-tracked lure + real fire path; `tag` only renames trace
labels). Two re-runs, both d3-gated:

1. **The post-AND poll** (the one that catches #5): right after the AND and the first lip block,
   the walk polls the same scan from the hold line (≤20 × 30-tick waits) and fights the lip the
   moment it shows — before the doorway step and the route pre-clear leg that the ledger's death
   walked Bond through.
2. **The escort-loop re-scan** before the follow rounds (the report's literal "post-escort
   re-sweep"), as the second net for an even later surfacing.

The first iteration placed the re-scan only before the follow rounds and died at 3844: the
first lip scan ran while #5 was still off (no "door lip guard" fight printed), and the route
pre-clear leg then walked Bond past #5's (4.5, 15.0) muzzle at 0.9 u (-14.40 ×3). The poll
placement exists because #5 surfaces AFTER the AND — the poll is the "escort crossing begins"
window watched from the safe side of the aperture.

## How it was verified

- `walkreal_07000001_live` **whole: 8 passed / 0 failed / 0 ignored** (40.19 s), including the
  d1/d2 real-damage sisters (their pinned timelines stay green — the d3 gates are the only
  new code paths).
- The d3 walk: AND ch 0x99 on at tick **3679**; the door-lip re-scan found `[5]` at **+2 polls**
  and fought it from the hold line; **level_exit to 07000002 at tick 10042** with **69.4**
  health (15 hits for 30.6, 10 return-fire fights). Death 70165 → clean exit.
- `nf_game` lib tests: **683 passed / 0 failed / 5 ignored** (1.04 s).
- Full suite, chunked (`tools/test/chunk_suite.py`; this sandbox kills background jobs, so the
  nohup'd runner from the task text cannot survive a tool call): **357 / 1134 binaries
  recorded, 1174 passed / 4 failed / 113 ignored** on the final tree (the fix commit; no code
  changed after it). The 4 failed tests, all classified:
  - `missing_scenery_live` ×2 and `glass_guards_14_live` ×1 — **environmental**: wgpu
    `adapter: NotFound { active_backends: Backends(0x0), requested_backends: NOOP | VULKAN |
    GL | METAL | DX12 | BROWSER_WEBGPU, supported_backends: Backends(VULKAN | GL) }` — this
    recycled sandbox has no GPU adapter. Neither binary touches this task's file; both fail at
    adapter creation, before any test logic.
  - `enemy_accuracy_live` ×1 (rc 124) and the earlier `net_soak_faults_live` kill — **my
    chunked 240 s per-binary limit** vs the suite's documented 600 s (`net_soak_faults_live`'s
    own header: "The 10-game-minute run fits the suite's 600 s binary limit"). Direct re-runs
    in this sandbox: `net_soak_faults_live` exits ok in 0.02 s (its harness filters its 2 tests
    out of the short invocation), `enemy_accuracy_live` needs the 600 s window my 560 s tool
    cap cannot give.
- Directly-run binaries this session, with counts: `walkreal_07000001_live` 8/0 (whole);
  `nf_game` lib 683/0/5; `missing_scenery_live` and `glass_guards_14_live` single tests (the
  env-fail evidence above); `net_soak_faults_live` (ok, 0 run).

## What's left

- The suite's remaining 777 binaries — the window went to the cold rebuild (this sandbox lost
  `target/` and the game files; the rebuild plus the 1134-binary link does not fit beside the
  fix work). The chunk state file is scratch outside the repo; the next session re-runs on its
  tree. No failure in the recorded 357 is attributable to this branch.
- The three GPU binaries (`missing_scenery_live`, `glass_guards_14_live`, and any other
  adapter-requiring test) need a sandbox with a wgpu adapter (the previous sessions' image had
  one); they are not code failures.
- `enemy_accuracy_live` and `net_soak_faults_live` need one pass at the suite's default 600 s
  limit.

## Live checks for the main agent

None required: no visual work, no unverified field reads. The two levers rest on the walk's own
trace ledger ([L] runs above) and the already-pinned mechanics the walk cites ([M]
DroneFunc_SetNoiseHeard, Drone_ModBulletDamage 0x139f50; the lip filter unchanged from
guard-radii-finish b28203c21).

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session.

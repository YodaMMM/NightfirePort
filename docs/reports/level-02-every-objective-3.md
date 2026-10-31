# level-02-every-objective-3 — report

Task: make every objective of 07000002 complete on real input in its walkreal test — settle the
two real-health blockers the previous round left, fix the port gaps, and un-ignore what can be
un-ignored. Branch `s/level-02-every-objective-3`, from `origin/s/level-02-every-objective-2`
5270c0e9 merged with origin/master 7d0091a0.

## The merge

`origin/master` had moved past the e2 branch: `fd649f0f` (level-02-captors-real-approach, Oct 30)
rewrote `clear_captors` to walk the nav web into the captor room — a dedicated follow-up to e2's
report, NEWER than e2's vantage approach. The one conflicting file
(`walkreal_07000002_live.rs`) took master's side in both hunks (clear_captors) and kept e2's
`loot`/`clear_pursuers`/real-health legs. Note: master's own copy of this test is the OLD
stand-in walk (god-mode, no fight legs) — fd649f0f's "1 passed" in its commit message is that
master route with the teleport removed, not the real-health walk.

## What this round did

1. **Reproduced blocker 1 deterministically** (run1, 2026-10-31, NF_PICKUPLOG=1): the real-health
   walk dies exactly as e2 documented — during `south hall west nav 10` the fail chain fires and
   `level_exit` is set at **+1462 ticks** (the auto-restart answering the fail menu the walker
   never does [M scene.rs → fail_choose_restart 0x216618]); every later leg returns +0 on the
   level_exit rails and the ch-0x17 assert fails.
2. **Fixed the harness gap (the walker had no drone-body-block response).**
   `crates/nf-view/tests/common/walker.rs`, the stuck ladder of `walk_to`: when a leg makes no
   progress for 60 ticks AND a live hostile (team 1 — the same filter the walkreal fights use)
   stands within 1.8 u on Bond's own floor, the ladder now answers like the player does:
   - walk down to a different-floor blocker first (|dy| > 0.7) and fight at eye level;
   - `shoot()` — the real fire path (exposed-part aim, tap discipline, flank);
   - when NO part is exposed at body range (< 3 u), STEP BACK away from the guard first
     (distance opens the vertical angle) before the perpendicular flank;
   - on a dry clip, `resupply` (the walked-path corridor loot);
   - a `fighting_blocker` guard keeps the arm's own nested walks (shoot's flanks, resupply) from
     re-triggering the arm (run2's recursion).
   All real input: walk + fire + E, no position writes, no channel writes.
3. **The g3 wedge is GONE.** Runs 2–5 all pass `south hall west nav 10` (the arm engages the
   blocker there; run2 log: `hostile body-block (drone 3) — shooting the blocker`, then the walk
   continues to the west wing).
4. **The real takes WORK on this tree** — e2's "the reachable drops … the arrive-0.4 take still
   did not grant" half of blocker 2 does not reproduce: `[take] tick 3474 weapon 0x12 kind 0
   rounds 11 loaded true → Some(Weapon { weapon: 18 })` and `[take] tick 6722 weapon 0x1c kind 0
   rounds 7 → Some(Weapon { weapon: 28 })` (NF_PICKUPLOG, run2/run3) — two real pickup takes
   through Bond's own capsule mid-fight, exactly the loot loop's job.
5. The **y 0.0 through-floor drop** (e2 run08) did not reproduce in runs 1–5 (no looting line
   below y 4.4). e2's run08 happened AFTER the fail-restart had already fired; the drop economy
   needs a re-check once the remaining blocker below is settled.

## Where the walk stands (honest)

Still `#[ignore]`d — the un-ignore is not honest yet. The remaining blocker, pinned with this
round's evidence:

**The west-wing lip fight.** A late-closing ch-0x15 guard (drone 4 — the fifth garden pursuer;
the other four die to the route's sweeps) catches Bond at `north of the ch-0x17 plate nav 5`:
~1.4–2.7 u off with **NO exposed part** (`exposed false`, `no exposed part for 540 ticks` —
every part's eye→centre segment clips something at that angle [L run3/run4]). The step-back and
flank walks route through the nav graph and wedge at the 3000-tick lenient caps (the west-wing
nav routes over the south-hall arch: bond y 7.03 on floor_below 5.34, run4 dump). Bond took ~70
damage standing in the open (health 29.5, run4). The legs still complete and the walk reaches the
geisha asserts — but on the damage budget the later fights need.

**Next blocker after the lip (new this round):** assert 241 — `the enabled geisha sits in
Hostage 0x0A` — failed in runs 3–5, with NO fail-restart involved (run4: no level_exit line
anywhere; Bond alive at 29.5). Either the port moves the geisha out of Hostage while combat runs
nearby, or the captor's own state machine drags her — this needs eyes before the walk can assert
it. See the live checks.

## What ran (totals)

- `walkreal_07000002_live --ignored` (the touched binary, direct, five runs 2026-10-31):
  0 passed / 1 failed / 0 ignored — the failure moved from the nav-10 fail chain (run1, +1462)
  to the lip fight / geisha assert (runs 2–5). The walk's progression each run is in
  `/tmp/w02_run*.log` (sandbox-local; the quotes above are the evidence).
- `interactables_07000002_live`: **16 passed; 0 failed; 0 ignored** (14.14 s).
- `nf-game` (whole crate, `--test-threads=1`): **953 passed; 0 failed; 9 ignored** (RC 0).
- **The full suite: no total line this window.** The fast runner was started in the first
  minutes per the task, but this sandbox kills background jobs between tool calls (the e2 report
  documented the same); `tools/test/chunk_suite.py` on the FINAL tree recorded 3/983 binaries —
  **100 passed, 0 failed, 0 ignored** (nf_view lib 96/0 among them) — before the window closed.
  Resume with:
  `NF_ISO=<abs> CARGO_TARGET_DIR=$HOME/NightfirePort/target python3 tools/test/chunk_suite.py /home/z/suite02final_state.json`
  (state file committed? no — sandbox scratch; the command recreates it) and run to ALL DONE.

## Live checks for the main agent

1. **The geisha state during nearby combat.** On a fresh 07000002, enable the ch-0x17 group
   (stand on the plate at (−35.2, 6.5, 5.05)), keep a guard alive within ~10 u in combat, and
   watch geisha 1's state word (the port's `drones.list[gi].state`; the game's Drone state field
   at +0x8 per docs/fields). Expected per my reading of the hostage binding [M 0x144030/0x1441c0]:
   she HOLDS Hostage 0x0A while the captor lives. If the EE moves her (flee/down), the assert is
   wrong and must follow the game; if the EE holds her, the port has a gap in the hostage state
   hold during combat.
2. **The drone-4 lip.** A render/inspect at `--bond -41.8,6.45,8.65` facing (−35.2, 6.4, 11.5):
   where does the fifth garden guard stand relative to the stair lip (crouched behind the slab?
   on the lower floor?), and which hit boxes does a player actually see? My `exposed_aim` says
   none — if the live game shows a clearly shootable part, the port's hit-box enumeration or the
   eye anchor is the port gap to fix.

## What's left

- Settle the lip fight (the walker needs a workable real-input move against a no-exposure
  close guard — options: a real down-the-step approach to his floor when |dy| is small but the
  lip blocks, or leading him around the pillar), then the geisha assert (live check 1).
- Re-check the drop economy (the y 0.0 drop) after the walk survives the west wing.
- Un-ignore and take the walk green with NO stand-ins; the route and legs are already real.
- The full-suite total line on the final tree (chunk_suite, the command above).

## Usage

not visible to me (no token, cost or context-window figures are exposed to this session; the only
proxies visible are the shell's wall-clock timestamps: setup/merge 06:37–06:45 UTC, the reproduce
and fix cycles 07:00–08:00 UTC, the report 08:20–08:35 UTC 2026-10-31).

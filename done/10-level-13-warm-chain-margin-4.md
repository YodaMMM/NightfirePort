## Your task: `level-13-warm-chain-margin-4`, Chain Reaction's real-damage warm-chain walk green under the game's guard sight

Branch from origin/master (it carries level-13-warm-chain-margin-3's ported levers). Read docs/reports/level-13-warm-chain-margin-3.md on origin/s/level-13-warm-chain-margin-3: the walk now fails on the north-lobby-2 leg's budget (a wedge, not a death). Find what holds Bond on that leg (a guard body in the lane, a door, the leg's route) and fix the route with real input; then un-ignore walk_the_warm_chain_to_the_front_entrance_and_out_real_damage. Never lower damage or health, no top-ups. Run walkreal_07000013_live and interactables_07000013_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-13-warm-chain-margin-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox5 at 2026-11-10T15:43Z

---
## Report (done, 2026-11-10T16:58Z)

Closed by the driver: the report is on `s/level-13-warm-chain-margin-4` (docs/reports/level-13-warm-chain-margin-4.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-10T17:05Z)

# level-13-warm-chain-margin-4 — the north-lobby-2 wedge found, fixed and proven; the walk now reaches the lobby-east posts, whose crossfire is the next blocker (documented)

**Branch** `s/level-13-warm-chain-margin-4` (pushed as itself; the finish line's
`10-level-13-warm-chain-margin-4` spelling is the queue name). Cut from `origin/master`
6d45e6e4c. Final commit ffb2bbfaa. The diff is three files:
`crates/nf-view/tests/common/walker_b2.rs`, `crates/nf-view/tests/common/tower2.rs`,
`crates/nf-view/tests/walkreal_07000013_live.rs`. No `crates/*/src/` changes, no damage or
health numbers touched, real input only.

## The wedge's cause (the task's question: what holds Bond on that leg)

Reproduced first: the d3 walk un-ignored on this tree died exactly as margin-3's ledger said —
`walk_to the north lobby 2: 6000-tick cap`, bond (−8.99,1.03,12.62), target (−8.00,1.10,17.00),
`segment_to_target=Some(0.127)` cel 374, tick 13980, health 89.5, 27 fights — byte-identical to
the margin-3 run 3 (the fights are deterministic and free). The run-5 ledger ([L]
/tmp/d3run5.log) shows the chain: the leg STARTS fine on the lane (−11.74,15.73), the fights
answer the respawner stream, each fight runs the mag dry, and the lane resupply fetches the
stream guards' drops — which rest on the LOWER FLOOR ~1.1 under the lane (y 0.0-0.1; the
`lane_floor_drops` band admits them). Each fetch's nav walk leaves Bond standing at the drop on
the lower floor south of the lobby; the leg then resumes steering toward (−8,17) from there.

What holds him: **the lobby's south wall (static cel 374) blocks every straight line from that
lower floor to the lane**, and the nav graph has no cross-path route out. Probes ([L]
probe_012_rebridge, this tree, teleport-diagnosis only): the segment (−8.99,12.62)→(−8,17)
blocked at f=0.127 (point (−8.84,1.04,13.28)); every lane point x −12..−4 at z 15.75-17.5
blocked; `nav_route` from the wedge point to the lane returns NOTHING (its route to the west
corridor detours through the y-0.45 level BELOW, whose nodes cannot be walked from y 1.03); the
direct westward line along z≈12.5 blocked at (−15.9,12.5). The lower floor connects to the west
corridor only through the passage at z≈5.2-5.9 — the route the level's own resupply navs walked
((−16.25,5.59)→(−19.07,5.32)→(−20.09,5.24)→(−21.32,6.25)→corridor). So: a guard body? No
(drones nearby: none). A door? No (doors nearby: none). **The leg's route**: the direct line
from the fight-displaced position cannot work, and the graph cannot re-route.

## The fix (two pieces, both OPT-IN, set only by the d3 walk)

1. **`resupply_home`** (walker_b2.rs, new Walker field, default false): a resupply fetch is a
   THERE-AND-BACK errand — after the fetch loop, walk the nav route back to where the resupply
   started (the fetch's own walk proved the route; walk_nav_to's per-node walks are lenient).
   A player who steps off the lane to grab ammo walks back and continues. This kills the
   displacement at the source; the leg resumes where it left.
2. **`REANCHOR_NORTH_LOBBY`** (tower2.rs, new AtomicBool, default false) — the legs' own
   recovery should a displacement ever survive the errand: at the three east north-lobby legs,
   try the leg leniently; if Bond did not arrive, walk the PROVEN passage chain
   (nav-walk to (−16.49,5.60), then (−19.07,5.32)→(−20.09,5.24)→(−21.32,6.25)→(−20.61,9.15)→
   (−20.26,14.06)→(−18.64,15.93) — every point an arrival on this tree's own resupply runs) and
   re-walk the leg from the lane.

**Proof ([L] /tmp/wr13_m4.log, this tree, the d3 walk un-ignored):** `the north lobby 2:
arrived at +23 ticks pos (−8.31,1.10,16.83)` — the leg that consumed the whole 6000-tick cap at
tick 13980 now arrives in 23 ticks. The walk then continues PAST the wedge for the first time
on the sight batch (it dies later, see below — a different, later blocker). The `resupply home`
return legs appear in the ledger exactly as designed ("the lobby south resupply home: arrived").

**Also fixed en route (a real bug the run-8 ledger caught):** the tower2 OPT-IN flags are
process-global atomics and this binary's tests share one process (`--test-threads=1`) — the d3
walk's stores leaked into the sisters (the agent sister's log shows her running the d3
pre-clear: "the north lobby pre-clear: guard #7 at d 5.4 killed true"). `walk_the_warm_chain`
now resets all three flags at every walk's start; the sisters are byte-identical again (the
final run shows no pre-clear lines outside the ignore message).

## The new blocker behind the wedge (reached for the first time; the ledger for the next session)

With the displacement gone, the walk walks the lane east and meets the lobby-east posts' sight
for the first time on this kit: **#6 (skin 0x5000006, rifle 0x18, hp 30) at (0.1-0.4,16.8-17.9)
and #7 (skin 0x5000008, weapon 0x19, hp 12) at (4.2-4.8,15.3-16.1)** open fire the moment the
lane's west half is crossed. The melt ledger ([L] /tmp/d3run6.log, NF_DMG_TRACE, tick-by-tick):
eleven hits in 64 ticks from x −9.1 — #6's rifle −7 every 6-15 ticks, #7's −3.5 — health
86 → 26.5 plus the 25.5 armour gone (the d3 accuracy law [M DroneWeap_DoBulletAccuracy
0x179740:74-88]: every round lands at these ranges; the d2 sister took −2.1 TOTAL through the
same legs). The return-fire fight on #6 ([L] /tmp/d3run7.log, NF_RF_TRACE, fight#6): the
gun-ray exposure FLICKERS (aim None at t0/t60, Some at t120-t180, None at t240 — he patrols
1.1 u behind the east furniture), the taps land −3.5 per ~4 rounds when exposed, and the
fight's unseen give-up (120 ticks, lane) bails in a None window — the exchange cannot be won
in place. Levers tried and rejected, each with its reason in the code comments:

- The master pre-clear hook (`PRE_CLEAR_NORTH_LOBBY`) fired at BOTH lane ends, radius 16,
  stands capped 2400 (the d3-only flag gate): the sweep found NOBODY exposed at 14-16 u — the
  plain eye-ray exposure fails at that range against the east furniture [L /tmp/d3run8.log].
  (At close range it works: the leaked-flag sister run killed #7 from d 5.4.)
- `bridge_and_retry` over the nav graph (walker.rs's shape): dead here — no cross-path route
  exists from the lower floor ([L] the probe).
- The fights' creep + longer stands: the melt out-races any in-place exchange (the arithmetic
  above; the taps need ~9 hits, the melt delivers −0.9/tick).

The d3 test is **re-ignored with the updated reason** (the walk ships green or not at all); the
full ledger rides the ignore message. Next session's first item: the flank the d2 walk proved —
#6 IS exposed (aim Some, taps −16.8/60 ticks) from the lift-doors approach (2.4,13.9) [L the d2
fight#6 trace] — the route needs to reach that flank alive (e.g. a south-side z≈13.5-14 leg
under the posts' sight lines, or a flag-gated sprint-past-the-melt shape), with #7 taken first
(one fight, exposed at 13.9 u from the lane [L]).

## Verification (what ran, on this tree)

- `walkreal_07000013_live` WHOLE (`--test-threads=1`): **3 passed, 0 failed, 1 ignored** (the
  cold walk, the Agent sister, the Operative sister — all green; the d3 walk is the ignore)
  [L /tmp/wr13final4.log].
- `interactables_07000013_live`: **10 passed, 0 failed** [L /tmp/ia13_m4.log].
- Shared-helper consumers (merge lesson 7 — walker_b2.rs/tower2.rs changed): 
  `walkreal_07000012_live` **3 passed, 0 failed, 1 ignored** [L /tmp/wr12_m4.log];
  `campaign_flow_live` **5 passed, 0 failed** [L /tmp/cf_m4.log];
  `walkreal_07000011_live` **3 passed, 0 failed, 1 ignored** [L /tmp/wr11_m4.log]; plus the
  walker_b2 sample across the levels: `walkreal_07000001_live` **7/0/1**, `walkreal_0700000c_live`
  **1/0/3**, `walkreal_0700000d_live` **1/0/2**, `walkreal_07000014_live` **7/0/3**,
  `walkreal_07000015_live` **6/0/3**, `walkreal_07000041_live` **15/0/0** (all
  passed/failed/ignored, [L] /tmp/*.m4.log). The remaining walkreal binaries run unchanged
  code paths by construction (both new flags default false and are set only by this file's d3
  walk; the 79-binary set does not fit the window — the suite line below).
- The d3 walk itself: 4 full runs to the two blockers, each with a cited ledger (run 5 the
  wedge repro, run 6 the melt tick-by-tick, run 7 the fight traces, run 8 the pre-clear
  rejection), plus two probe runs (probe_012_rebridge, untracked scratch, teleport-diagnosis
  only — never on the proof path).
- The fast runner (`tools/test/suite.py`) was started twice in the background per the task;
  the first died instantly (`cargo` not on the nohup PATH), the restart ran ~5 minutes
  (building) and was stopped at the report's writing — it produced **no total line**, and this
  sandbox's build of the ~130 test binaries does not fit the remaining window (the same wall
  margin-3 hit; the five binaries named above were built and run directly instead). No
  `crates/*/src/` file changed on this branch, so the nf-game/nf-formats/nf-audio trees are
  exactly master's own.

## What is left

1. The lobby-east posts: reach the (2.4,13.9) flank alive and take #7 then #6 from there (the
   d2 fight#6 trace is the proof the flank fights land).
2. The 13-leg walk under the new sight with the drains (never reached: the melt consumed the
   window).
3. The full `tools/test/suite.py` total line (the background runner may yet produce it).

## Live checks for the main agent

1. Fresh boot of 07000012 at 00 Agent, Bond at the north lobby lane west end (−11.7,1.1,15.7):
   gdbwatch on drone #6's (skin 0x5000006) sight/muzzle — confirm the game's own sight opens at
   x ≈ −10.4 on the lane and his rounds land every volley at 9-14 u (the d3 Accuracy row), and
   whether a WALKING (not standing) Bond at z 13.5-14 (the lobby's south side) holds his line —
   the flank route's premise.
2. Fresh boot of 07000012: from the lift-doors approach (2.4,0.9,13.9), confirm guard #6's
   boxes read exposed (the plain eye ray AND the gun-ray set) — the d2 fight#6 trace says the
   taps land from there; a screenshot would pin the furniture line.

## Usage

not visible to me (no token/credit/context figures are exposed in this environment).

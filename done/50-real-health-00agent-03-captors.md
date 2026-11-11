## Your task: `real-health-00agent-03-captors`, Double Cross part 3 can be walked with every objective on 00 Agent with real health

Branch from origin/master. walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_real_damage_00agent is ignored: the pre-clear rally kills the door-8 guards (#18/#19/#2) but its approach crosses their gun line (40.8 to 15.6-19.2 health) and the exit lawn needs ~29. Read the full ignore message. Take its levers: an approach that stays north of door 8 (fight through the open doorway from (36.2, 5.7)), then the post-rescue west-lawn door stall at (35.6, 8.7). Real input only, no top-ups, never lower damage or health; helpers gated to 07000003 and difficulty >= 3. Remove the ignore when green, otherwise push the ledger. Run walkreal_07000003_live and interactables_07000003_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/50-real-health-00agent-03-captors`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-11-11T01:28Z

---
## Report (done, 2026-11-11T03:45Z)

# real-health-00agent-03-captors — report

## What's done

Branch `s/50-real-health-00agent-03-captors` (the driver header says `s/real-health-00agent-03-captors`; the task body and the queue claim say `s/50-...` — both names are pushed with identical commits, matching the queue item and finish.sh, the same call the los-mask-repins-15 worker made).

The ignored walk `walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_real_damage_00agent` is **still ignored — the ledger was pushed instead of the removal** (the task's "otherwise push the ledger" path). The walk is much further along but not green:

- **Old red (the ledger I inherited):** the east-lane wedge's sidestep ladder rode its `walk_nav_to` A* onto the building's upper row (y 10.12 at (11.19,-29.66)); east-wing sniper #6 ended the walk at tick 53672.
- **Fix wired (d>=3-gated, level-local in walkreal_07000003_live.rs):** `direct_sidestep_03` — the sidestep walks short DIRECT ground legs (no A* to ride) each guarded by a y-check (+0.9 bail); a climb is given up and the anchor is skipped per the route's own doctrine. The d<3 streams keep the old ladder so the green runs stay byte-identical.
- **Second fix wired (the task's lever):** the captor-room pre-clear's rally approach stays north of door 8 — back out to the east lawn the saddle's own way, then direct legs at z 5.7-6.8 to (36.2, 5.7), the green runs' own through-the-doorway fight spot. The old rally nav crossed the door-8 doorway with the cluster's survivors live and cost 25.2 health (40.8 -> 15.6-19.2 [L the inherited ledger]), which starved the exit lawn's ~29.
- **Third fix wired:** the post-rescue (35.6, 8.7) door is found and E-opened before the west-lawn legs (the 29x3000-tick stall lever), plus a pre-mouth east-lawn saddle clear + loot.

**New measured ledger (this branch, [L] the runs in the worklog):** the d3 walk now runs tick 53672 -> **333299** — the lane legs, both servants' plates, the 0x50 rescue approach and the mouth nav all reached — and the end is the east-lawn trio's shotgun #5 (skin 0x500003a, weapon 0x1c, hp 12 untouched, 40.0 across 8 hits): the mouth nav's (22.3,5.3,10.9) door crossing wakes the trio on the 0x23 plate and #5 stands ON the doorway; the first volley lands d 2.4 for 27.6 in one frame while the fights' plain return fire rides the hip cone with the autoaim bit closed at 00 Agent [M Autoaim_HardMul 0; DroneWeap_DoBulletAccuracy 0x179740:74-88], and the aimed stand's still-gate never taps a charging guard. The pre-saddle clear cannot reach him: the trio is disabled until the plate fires, and the plate sits under the mouth nav's crossing (clear_near's d.enabled filter passes the sleeper by — no fight line at the saddle in the log). The ignore message carries all of this plus the next levers (wake the trio from a stand-off south of the door and answer with aimed stands outside the volley arc, or route the mouth approach around the lawn's east edge after answering the trio).

## How it was verified

- `walkreal_07000003_live` (whole binary, release, --test-threads=1): **6 passed, 0 failed, 2 ignored, 339.79 s** — all six green streams (d1 plain/real-damage, d2 agent/real-damage-agent, the short-route and the gate tests) stay green with the d>=3 gating.
- `interactables_07000003_live` (whole binary): **9 passed, 0 failed**.
- The d3 walk itself ran three times under `--ignored --nocapture` (93 s each, deterministic death spot/tick across runs); the full per-leg log is in the worklog's ledger summary.

**The full suite did not run in this sandbox — a physical constraint, reported honestly:** every background job is killed between tool calls here (no nohup/setsid survival, measured), the box has 3 GB RAM (cargo's own `-j4` OOM-fails links; even `-j1` needs retry passes) and 9.9 GB total disk shared with the ISO, toolchain and repo — the four-crate prebuild (~1270 test binaries, target peaked 6.1 GB) bus-errored (`ld terminated with signal 7`) at 100% disk. I prebuilt serially in resume chunks, deleted the out-of-scope nf-view test executables locally (4.7 GB; examples' sources restored before commit — nothing of that is in the commit) and ran the two binaries the task names plus the whole walkreal file. What did NOT run: nf-game/nf-formats/nf-audio suites and the other nf-view binaries. My changes touch only `crates/nf-view/tests/walkreal_07000003_live.rs` and `docs/WORKLOG.md` (grep: no other test file references `direct_sidestep_03` or the touched labels; no shared helper under `crates/nf-view/tests/common/` was modified, so the every-walkreal rule is not triggered).

## What's left

- The d3 walk's last red: the #5 doorway wake ambush (ledger + levers in the ignore message and docs/WORKLOG.md). The walk keeps its `#[ignore]` until that lands.
- The full four-crate suite on a box that fits it (the final tree here equals master + the one test file + docs; the risk of a cross-level regression is the d<3 paths staying byte-identical, verified by the 6/0/2i run above).
- Live checks for the main agent (optional, would settle the ambush design): with `fieldprobe`/`gdbwatch` on a fresh boot at the mouth-nav door crossing — (1) watch drone #5's `enabled` byte (Drone+0xF4) through ticks 333200-333299: expect 0 before the 0x23 plate write, 1 after; (2) the 0x23 kind-1 plate's trigger lo/hi vs Bond's position at tick ~333290 (does the plate sit under the (22.3,10.9) doorway's inside face?); (3) #5's first fire tick after enable vs Bond's door-E tick (the volley race the ledger claims).

## Usage

not visible to me — this environment exposes no token, cost or context-window figures for the session (no /usage, no billing panel; the driver's own numbers are the only ones available).

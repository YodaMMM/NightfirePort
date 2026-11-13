## Your task: `real-health-00agent-01-hall-3`, 07000001's hall door phase can be won on 00 Agent with real health

Branch from origin/s/real-health-00agent-01-hall-2, merge current origin/master, and read its docs/reports/real-health-00agent-01-hall-2.md plus the 00 Agent ignore message. Take its next lever, the door-phase kill path. Either work out from the decompile which weapon and timing make the hearing chain draw the guards across [M DroneFunc_SetNoiseHeard 0x149920, HandleSoundAlerts 0x148a30] and lure with that, or kill the crowd through the aperture with crouched taps along the flight lane, keeping the ammo pool metered. Do not repeat its measured dead ends. Helpers stay gated to 07000001 and difficulty 3, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000001_live binary and interactables_07000001_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-01-hall-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
Main agent, fresh deaths on master 729f9694e (each ignored walk of the level, --ignored --test-threads=1; test names not captured):
walkreal_07000001_live: Bond died at tick 6709 (frame 6738) at (6.62,6.47,9.85) armour 0.0

---
claimed-by: sandbox4 at 2026-11-13T02:52Z

---
## Report (done, 2026-11-13T04:03Z)


---
## Report from the worker (done, 2026-11-13T04:03Z)

# real-health-00agent-01-hall-3

Branch `s/real-health-00agent-01-hall-3` (cut from `origin/s/real-health-00agent-01-hall-2`
ec54db62c, merged with origin/master — merge ca17d3e5b; one conflict in
`crates/nf-view/tests/common/walker.rs` resolved by keeping BOTH hall-2's `flank_hold_z` field
and master's `record_inputs`/`record_states` replay machinery).

## What's done

The task's lever — the door-phase kill path — was worked on BOTH arms the task named, and both
are now measured to their bolts. The 00 Agent walk is still NOT green; the ledger + the next
lever are pushed in the d3 test's ignore message, and the WIP machinery (the lane grid, the
diag counters, the peek dance) is on the branch, d3-gated, with d1/d2 byte-identical.

1. **The hearing-chain (lure) arm is dead by the AI's own gate.** The door-phase crowd is
   sighted-attacking; the sighting flag `+0x4F8 0x4000000` never clears on 07000001 (the only
   clear site is the 09/0A tower-impact path [M `drone.rs` `tower_impact_stamp`]), and
   `DroneVision_AlertSound` 0x177800 returns 0 on a sighted drone for msg SOUND — no weapon or
   timing draws a crossing once the crowd has seen Bond [M `drone.rs` `enemy_alerts`]. The
   diag lure run measured it: the P2K (0x403 — weight 65, outer 40 [L SFXOutputData
   0x2c94a0+0x18·0x403, EE snapshot]) fired 56 taps, the SMG 0x12 (fire sound 0x3 — weight 50,
   outer 30 [L the same table]) 55, the fists 56 — `alive true across true` ×3, healths
   12.0/21.2/12.0 untouched. The hall-1 lure that worked carried the SMG on a stream where the
   crowd was NOT yet sighted; that window never opens on this stream.
2. **The metered-tap arm is dead from the hold line.** The same diag run landed **0 of
   56/55/56 taps**: the flight lane (`segment_flight_scoped`, the corridor pre-clear's own
   probe [M drone.rs:16465]) to every guard's anchor was blocked on EVERY trigger tick. The
   flight-lane grid run (d3-gated, read-only, both eye heights) mapped the whole hold area:
   64 clear (cell, guard, height) combos — the middle band (x 6-7.5) that holds the deep line
   is walled at BOTH the crouched and the standing eye; the clear cells sit only in the west
   band (x 4-5) and the east band (x 8-9) — and BOTH wedge the nav (the walk stops 1.6-1.7 u
   short of (5,8.5)/(8,6), `segment_to_target` 0.32-0.92 [L the stance-fight run]).
3. **The crouch hides Bond.** The crouched peek dance (4 windows × 3 guards at
   (7.4,9.5)/(7.3,9.3), standing after the first design iteration) drew NO advance across 24
   windows — the guards held z 13.5-19.4, Bond took 0 [L the peek runs].
4. **The mechanic that DOES work** [L the stance-fight run, window 2]: a STANDING walk through
   the door lane (the nav toward (9,10), crossing z 9.3+ standing) draws the crowd's advance —
   guard 6 advanced to the door line and DIED to the walk's own return fire at tick 16518 for
   3.6 hp — but a nav leg LINGERS, and the deep captain #7 (0x1c, 10.3 u) lands his 5-pellet
   7.20 volley (36.0 in one frame); Bond died at tick 16593. The 16518→16593 window is the
   measured lethal linger.

**The next lever** (in the ignore message): the door-lane advance dance with a hard exchange
budget — walk INTO the lane standing (the fight3 shape), let ONE exchange run, leave the lane
IMMEDIATELY on the first wound or kill; the wound ledger is 55.0 at the door phase, the
advancer's volley 3.6, the captain's 36.0 — one captain volley per window is the survivable
budget, so the captain's LOS windows must interleave with the deep-line breaks.

## How it was verified

- `walkreal_07000001_live` (whole binary, `--test-threads=1`): **7 passed, 1 failed** — the
  only failure is the d3 `walk_..._real_damage_00agent` ledger test; the d1 real-damage and d2
  agent sisters pass (every change is difficulty-gated; their streams are byte-identical)
  [L 99.4 s].
- `interactables_07000001_live`: **10 passed, 0 failed** [L 29.4 s].
- The d3 diag/measure runs cited above ([L], per-tick tap/lane/health counters in the run
  logs, /tmp/d3_*.txt on this sandbox).
- **The fast suite did not run**: `tools/test/suite.py` died the way hall-2 documented —
  background processes are killed between tool calls in this sandbox (three starts, one with
  cargo missing from the nohup PATH, then empty `suite.out` with no process); the explicit
  batteries above are the coverage. **Re-run the suite on this branch before merging.**
  Disk note: the merge + the RUSTFLAGS change forked `target/release/build/nf-view`
  fingerprints (4.2 G of stale outputs) — `cargo clean -p nf-view --release` was needed before
  anything built (ENOSPC at 650 M free).

## What's left

- The advance dance with the hard exchange budget (the next lever above), then the AND gate
  and the rest of the walk (the walk's post-AND route is untouched and was green through the
  AND on the pre-hall-2 streams' machinery).
- The suite re-run on the final tree.

## Live checks for the main agent (PCSX2, fresh boot)

1. The door-phase crowd's sight: with Bond crouched at (7.4,9.5) vs standing at the same spot,
   record which one the ch-0xE crowd reacts to (the crouched peek drew nothing across 24
   windows [L]; the standing walker drew the advance [L]). Expect: standing crosses the
   crowd's vision threshold through the aperture, crouched does not.
2. The captain #7's volley cadence with LOS from (8.8,6.6,19.4) to a standing target in the
   door lane: the measured kill window was 75 ticks between the advancer's kill and the
   captain's 5-pellet volley. Expected: the burst re-aim time ~1 s, so a peek shorter than
   ~60 ticks after a wound is out before the next volley.

## Usage

Token/credit/context figures: `not visible to me` — this environment exposes no token, cost
or context-window readout to the worker. What was visible: the task message's clock (sandbox
expiry ~05:09 UTC; this report written ~04:36 UTC), `git` hashes, test outputs, and process
exit codes. No input/output token counters, no credit balance, no dollar cost.

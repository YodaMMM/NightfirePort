## Your task: `drone-bump-collision-3`, the guard push without breaking the escort and party routes

Start from `origin/s/drone-bump-collision-2` (the decompiled push chain: cel-straddle pick [M Collide_Pick 0x1eb320:125,312-336], the intersect gates [M 0x1ebaa8], Jointy's 0x800 kind gate [M 0x1ec180:203-209], the arming NDrone2_DoCollision [M 0x149ec8:11-34] fed by PreDroneControl's per-tick near slots [M 0x148db0:35-48]) merged with `origin/master`. Port the arming exactly: a bot's capsule takes part in the player push only on ticks its own DoCollision returns 1 - including the near-player slot Drone+0x154[i] written ONLY on the bot's control tick, so a bot whose control does not run keeps its old slot value [M]. Find from the decompile which bots' control pass the game skips (cut-scene drone freeze, scripted holds, disabled drones) [M the control walk and the freeze paths]. Then the seven reds the push caused must be green: mayhew_hold_live, party_door_lock (x2), walkreal_07000007_live, probe_07_route, cutscene_timer_pause, walkreal_07000011_live - each either green or re-derived with a decompile cite; drone_bump_live stays green. Also port the capsule-bottom height from the anim height word ext+0xa0 [M DroneAnim_PreTransCallback 0x13b490:63-66] instead of the fixed 1.0. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/drone-bump-collision-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-28T05:02Z

---
## Report (done, 2026-10-28T06:23Z)

Closed by the driver: the report is on `s/drone-bump-collision-3` (docs/reports/drone-bump-collision-3.md); the worker didn't run finish.sh.

---
## Report from the worker (blocked, 2026-10-28T06:23Z)


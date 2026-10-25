## Your task: `level-07-guest-collision-3`, frozen party walkers, then land guests-solid

Start from `origin/s/level-07-guest-collision-2` (merge it first; its report docs/reports/level-07-guest-collision-2.md has the chain). The push matches the game [M Collide_Jointy 0x1ec180:229-266, NDrone2_DoCollision 0x149ec8, NDrone2_PreDroneControl 0x148db0:35-47]; two walks are still red:

1. **Frozen mission walkers (a player-visible bug, pre-existing on master).** On 07000007, drone[22] (vt 0x10, ch 23, team 3, skin 0x5000026) walks Rook Patrol Path_3 and freezes at (79.53, 44.0, -50.32) inside the west doorway (door at (80.0, 43.0, -49.9)); drones 20/21 freeze mid-path-4 at (75.0, -45.3)/(75.8, -45.9). Route live, not blocked, state 0x16 CIVILIAN_MISSION, position unchanged for 1800 ticks. Leading candidate: `request_anim` (drone.rs ~2295) returns early when `anim.state == anim`, so the per-tick walk request never restarts a finished walk sequence. Find the game's rule [M — CallAnim / the anim layer's re-arm while moving, the CIVILIAN_MISSION update 0x15d860 and the route follow] and fix it for every drone, with a test that a mission walker completes its path, plus sisters on guards' patrols.
2. `walkreal_07000007_live::walk_the_party_start_to_finish` green once the walker no longer parks in the doorway (or, if the game parks him there too [M], the walk takes the player's way around).
3. `interactables_0700000a_live::armour_pickup_taken_by_walking_over_it`: the script's 20 civilians at t109 cross the route; the dodge must clear the crossing civilian before the alarm-zone restart (~t1500), with real input only.
4. Full suite on the final tree merged with origin/master; report the total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-07-guest-collision-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-10-25T05:30Z

---
## Report (done, 2026-10-25T06:50Z)


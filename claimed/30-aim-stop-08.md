## Your task: `aim-stop-08`, guards stop dead to shoot on 07000008 too

Branch from origin/master. The guards' AimStandFire root stop (Drone::zero_root, Drone+0x38 [M 0x166328:117-123 -> DroneAnim_PreTransCallback 0x13b490], [L PCSX2 07000011 d2: guard 0x1896b50 stops dead on its 0x68 entry at frame 214]) is live per level: the zero_root_live list at the AIM_STAND_FIRE UPDATE arm in crates/nf-game/src/drone.rs. Add this level to that list and re-tune its walks against the stop (real input only; never lower damage or health, no top-ups; helpers gated to this level). Done = the level's walk binaries all green with the stop on, then the full suite.

Measured with the stop on 08: (1) walk_the_whole_mission_on_real_input_real_damage dies lasering the safe bolts while #5 stands at 2.8 u (80.8 in 29 hits); answering every guard on Bond's floor within 10 u before each bolt (real-damage walks only, the fullest gun up, then back to (160.0, 43.4, -117.0)) makes it green. (2) ..._real_damage_agent then reaches the Hind at 41.9 health + 50 armour (the route guards cost ~58: #11 11.2, #5 10.5, #14 8.4, #2 8.4) and dies at 201.7/448.5 copter damage with rockets left; the route's health budget needs re-planning (the level's armour/health pickups [D], fights from cover). Run walkreal_07000008_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/50-real-health-agent-06`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

**Branch:** `s/30-aim-stop-08`.

---
claimed-by: sandbox3 at 2026-11-13T01:48Z

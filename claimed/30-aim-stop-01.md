## Your task: `aim-stop-01`, guards stop dead to shoot on 07000001 too

Branch from origin/master. The guards' AimStandFire root stop (Drone::zero_root, Drone+0x38 [M 0x166328:117-123 -> DroneAnim_PreTransCallback 0x13b490], [L PCSX2 07000011 d2: guard 0x1896b50 stops dead on its 0x68 entry at frame 214]) is live per level: the zero_root_live list at the AIM_STAND_FIRE UPDATE arm in crates/nf-game/src/drone.rs. Add this level to that list and re-tune its walks against the stop (real input only; never lower damage or health, no top-ups; helpers gated to this level). Done = the level's walk binaries all green with the stop on, then the full suite.

Measured with the stop on 01: (1) following_mayhew_down_completes_the_level: the west-end pair #15/#4 stand at (-30.5, 6.3, 13.5) and fire the length of the hall at the waiting contact 28 u off (hit_by_drone 15 at t5708: 10.0 -> 2.5; #4 finished him at t10021); widening the west-end pre-clear band from 20 to 30 u (farthest first) makes it green. (2) walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent: at the hall guard-6 peek the walk_to the stand (7.8, 6.46, 10.2) wedges 0.72 u short at (7.40, 9.60) for its 3000-tick cap in the door mouth, and #7 (24 hp) trades 37.8 (9 hits) with Bond there (81.4 -> 43.6); the west hall then takes the rest (#3/#12 at (-30.6, 13.3)), death at t12116. 01 has no armour pickup - the hall approach has to cost less. Run walkreal_07000001_live, fix_mission_live.

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

**Branch:** `s/30-aim-stop-01`.

---
claimed-by: sandbox1 at 2026-11-13T01:32Z

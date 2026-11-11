## Your task: `level-02-castle-walks-hearing`, the 02 castle real-health walks back on under the hearing fold

Branch from origin/master. Guards and hostage killers now hear gunfire through the shared sound fold (guard-noise-attack-states: msg 0x14 routed through DroneVision_EnemyAlerts for CivilianDoorGuard, HostageKiller and KikoMission [M 0x160648:154-161, 0x15c728:69-85]). Live check [L PCSX2 fresh 07000002, ch 0x17 up, Bond 12.4 u south of the killer, 10 pistol taps]: the killer goes 0x8 -> 0xa3 -> 0x8 -> 0xa4 -> 0x8 -> 0xa5 -> combat (0x67/0x5c/0x59), the hostage stays 0xa (no execution), the two guards go 0x5 -> 0xa3 -> 0xa4 -> 0x62 -> combat; the port matches. Two walks went red with it and are ignored: walkreal_07000002_live::walk_the_castle_all_objectives_real_health (no longer crosses the doorway sensor, ch 0x81) and ::walk_the_castle_all_objectives_real_health_00agent (ends short of the three captor deaths). Re-tune both on real input only (no position, health or channel writes), the way a player would handle guards who now come to the sound of the gun. Do not change the hearing code.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/05-level-02-castle-walks-hearing`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

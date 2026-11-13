## Your task: `aim-stop-15`, guards stop dead to shoot on 07000015 too

Branch from origin/master. The guards' AimStandFire root stop (Drone::zero_root, Drone+0x38 [M 0x166328:117-123 -> DroneAnim_PreTransCallback 0x13b490], [L PCSX2 07000011 d2: guard 0x1896b50 stops dead on its 0x68 entry at frame 214]) is live per level: the zero_root_live list at the AIM_STAND_FIRE UPDATE arm in crates/nf-game/src/drone.rs. Add this level to that list and re-tune its walks against the stop (real input only; never lower damage or health, no top-ups; helpers gated to this level). Done = the level's walk binaries all green with the stop on, then the full suite.

Measured with the stop on 15 (four different breaks): walk_spawn_to_the_omega_sector_exit_agent and _00agent: the fence mine #447 shot from the threshold runs dry (clip 1 spare 0 by t1800) and the line through door 42 is blocked (segment hit at 0); _real_damage dies at t8147 (#10 one 16.25 hit at the far end, then #12 at 4.5 u); _real_damage_agent crosses at 16.2 health against its >= 25 margin pin (#10 36.9 in one hit, the world 24.7 in 7). Run walkreal_07000015_live.

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

**Branch:** `s/30-aim-stop-15`.

---
claimed-by: sandbox5 at 2026-11-13T02:18Z

---
## Report (done, 2026-11-13T04:00Z)

Closed by the driver: the report is on `s/aim-stop-15` (docs/reports/aim-stop-15.md); the worker didn't run finish.sh.

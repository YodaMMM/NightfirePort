## Your task: `aim-stop-11`, guards stop dead to shoot on 07000011, with the room drawn gate

Branch from origin/master. The guards' AimStandFire root stop (Drone::zero_root, Drone+0x38 [M 0x166328:117-123 -> DroneAnim_PreTransCallback 0x13b490], [L PCSX2 07000011 d2: guard 0x1896b50 stops dead on its 0x68 entry at frame 214]) is live per level: the zero_root_live list at the AIM_STAND_FIRE UPDATE arm in crates/nf-game/src/drone.rs. Add this level to that list and re-tune its walks against the stop (real input only; never lower damage or health, no top-ups; helpers gated to this level). Done = the level's walk binaries all green with the stop on, then the full suite. Also add 0x11 to the room_gate_live list in the same file (the drawn flag keyed on room visibility [L 0a/02 evidence in celgraph.rs]).

Measured with both on 11: walk_spawn_to_the_lower_floor_load_point_real_damage_00agent loses 20 to #5 at the spawn detour, then 40.0 in one hit from the armour room's UNARMED #1 (placement 122) as the d3 pre-leg 'the east wall north' walks onto him at (20.07, 47.9, 15.96) (a 9 u clear at 'the east jog 2' never saw him), and the lower floor's 0x19 trio #33/#34/#35 take the last 40 at (14.2, 24.1, -3.3). engage_range_07000011_live needs the stop on (its [L] 5.8-9 hold). Run walkreal_07000011_live, engage_range_07000011_live, start_corridor_guard_07000011_live, stairwell_guards_07000011_live.

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

**Branch:** `s/30-aim-stop-11`.

---
claimed-by: main agent at 2026-11-13T00:27:35Z

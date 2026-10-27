## Your task: `placement-matrix-classes`, every placement class built with the game's own matrix

`level-14-vent-grates` (on master) found that the 0xd9 ScriptPlayer placements build their object matrix through `control_create_object` 0x1344e0's param_4 == 0 arm, i.e. `View_RotTransMatrix` 0x1e5168 (ported as `placement::view_rot_trans_matrix`), not `RotMatrix` 0x1e2368 which the port uses for everything else. The two agree only when rx = rz = 0. For every other placement class that can carry pitch/roll (0x19 non-main-map statics, doors 0x2a/0x22, markers, pickups, ladders, breakables, lights, any creator that passes rot to control_create_object or builds its own matrix), pin the creator's matrix builder [M] and switch the port where it differs. Census the disc for placements with non-zero rx/rz per class [D] so the change's reach is known; test each switched class against the [M] rows. Yaw-only placements must stay bit-identical.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/placement-matrix-classes`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-10-27T13:30Z

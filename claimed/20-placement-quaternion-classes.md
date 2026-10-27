## Your task: `placement-quaternion-classes`, emitters and mounted guns from the placement quaternion

`placement-matrix-classes` (on master) pinned every placement class's matrix builder; two remain on the port's Euler fallback: the 0xf2 emitters (built inline in the dispatcher with `Quat_QuatToMat` 0x1e3e80 on the placement's +0x24 quaternion [M 0x1d04f0:474-478]; 16 of 69 disc rows carry pitch/roll [D census]) and 0x34 `GunImp_Create` 0x131f20:66-69 (same path; its 37 rows are yaw-only today). Parse the +0x24 quaternion [D placement layout], port `Quat_QuatToMat` exactly [M], build both classes from it, and pin the rows against the decompile; yaw-only emitters must stay where they are (test the census rows). `world_fx.rs` carries the inline note.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/placement-quaternion-classes`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-27T22:29Z

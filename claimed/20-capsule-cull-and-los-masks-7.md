## Your task: `capsule-cull-and-los-masks-7`, the capsule cull and LOS masks without the door-state filter

Start from `origin/s/capsule-cull-and-los-masks-6` (merge origin/master). That branch carried the `Door_IsClosed` filter on the doors' collision parts, which turned door/walk tests red. `door-state-collision-2` (on master) showed from the decompile that movement collision is state-blind: `Door_IsClosed` 0x1367b0's only caller is `Vision_Recurse` 0x1e82e0:51 (render portal visibility), and `Collide_PickObj` 0x1eae80 never reads the door state. Drop the filter (keep master's unconditional door extend), keep the capsule cull and LOS mask work, and get the full suite green, including walkreal_07000001/07000008/07000014, probe_07_route and ai_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/capsule-cull-and-los-masks-7`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-10-27T19:04Z

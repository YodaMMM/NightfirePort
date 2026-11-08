## Your task: `level-07-raw-slide-sweep`, the doorway raw-slide fix keeps every level's walkers on the game's paths

Branch from origin/s/level-07-actor-stuck-doorway-2, merge current origin/master, and read its docs/reports/level-07-actor-stuck-doorway-2.md. The fix removed the probe from undrawn walkers, matching the game's rule [M 0x139268:70-93], and the report warns that a level whose walker relied on the probe to stop at render geometry with no AI wall would now walk differently. Run every walkreal_*_live and npc_doors_live one level per tool call, and record each level's result in the report. For any walker that now leaves its path, find whether the port is missing an AI wall the game has there [D the level's AI wall set] and fix that, never bringing the probe back. Run fix_guard_misc_live, the route and door binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-07-raw-slide-sweep`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-08T10:48Z

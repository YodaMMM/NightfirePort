## Your task: `enemy-mission-kind8`, EnemyMission 0x15e1e8: the kind-8 mission paths

`NDrone2_DSTATE_EnemyMission` 0x15e1e8 and its kind-8 paths (`AssignAIPath(8)`) are not ported (drone.rs notes it). Find which levels place guards that use it [D placements: the dtype / mission path params], port the state and the path assignment from the decompile [M], and add a real-path test on a level that uses it: the guard follows its mission path. Sister: a guard without a kind-8 path never enters the state.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/enemy-mission-kind8`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-23T16:51Z

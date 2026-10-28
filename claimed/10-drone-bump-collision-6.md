## Your task: `drone-bump-collision-6`, the bot-side move answer at the 07 party hall's west mouth

Continue `origin/s/drone-bump-collision-5` (merge `origin/master`). Its walker wait-beat is in; its report names the remaining piece: the bot-side move change at the west mouth (the armed route walker and the door swing cell sharing the doorway; the shared-RNG phase the merge moved). Port what the game's bot does there [M its kind-8 route step, the drone-vs-door and drone-vs-player avoidance], then turn green party_door_lock_07000007_live (x2), cutscene_timer_pause_live, probe_07_route and walkreal_07000007_live's hall walk - never by disarming the push. Run the FULL suite and quote its total line; every red accounted for.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/drone-bump-collision-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-28T12:50Z

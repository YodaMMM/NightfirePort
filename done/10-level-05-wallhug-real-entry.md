## Your task: `level-05-wallhug-real-entry`, the 07000005 wall-hug plate is reached by Bond's real crawl, not a teleport

level-05-ridge-plate (origin/s/level-05-ridge-plate, docs/reports/level-05-ridge-plate.md) proved the route to the ridge plate through real input. The interactables test for #301 "WallHug_touchplate" (out 67, the ledge cut-scene watch) still teleports Bond into its box over the road (z 70.76..72.15 [D]). Replace the teleport with the real crawl entry: the prompt leg at (25.5, 30.96, 71.0) sits inside that box. Pin that the wall-hug starts from Bond's own approach, the plate fires out 67, and the cut-scene plays. If the real entry fails where the game's succeeds, find out why against the decompile and fix the port. Never lower damage or health. Run interactables_07000005_live, walkreal_07000005_live, spawner_kill_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-05-wallhug-real-entry`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-08T23:19Z

---
## Report (done, 2026-11-08T23:53Z)


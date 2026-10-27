## Your task: `level-0a-rain-indoors-5`, the last red from the rain fix: the 41 glass pane

Start from `origin/s/level-0a-rain-indoors-4` (merge it, then origin/master). On the merged tree the full suite has ONE red that is green on master: `interactables_07000041_live::the_glass_pan_shatters_under_the_real_fire` (the rain-3 report's second stream-shift red: a pinned fire window riding the shared generator's position [M Rand_* 0x1e36f0..0x1e3780]). Find which draw decides the glass break on that path (the fire spread, a damage roll, the glass health), and whether the test pins a game rule or one lucky stream value; make it assert the game's rule (or drive the real fire the way a player does), never weaker than [M]. Everything else stays green; run the full suite on the merged tree.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-rain-indoors-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox6 at 2026-10-25T18:42Z

---
## Report (done, 2026-10-25T19:36Z)

Closed by the driver: the report is on `s/level-0a-rain-indoors-5` (docs/reports/level-0a-rain-indoors-5.md); the worker didn't run finish.sh.

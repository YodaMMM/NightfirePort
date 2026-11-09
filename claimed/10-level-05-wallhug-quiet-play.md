## Your task: `level-05-wallhug-quiet-play`, the 07000005 wall-hug cut-scene plays when Bond arrives quietly

Branch from origin/s/level-05-wallhug-real-entry, merge current origin/master, and read its docs/reports/level-05-wallhug-real-entry.md. The wall-hug plate is now reached through Bond's real crawl, and the rule pair (refuse under combat music, play when quiet) is pinned. What's left is a real quiet arrival that plays the cut-scene: either walk in without waking the two out-of-line posters, or kill them silently from the slope side first. Pin that the cut-scene plays, the plate's channel writes happen in the game's order, and play resumes. Helpers stay gated to 07000005. Real input only, never lower damage or health. Run interactables_07000005_live, walkreal_07000005_live, spawner_kill_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-05-wallhug-quiet-play`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-09T11:59Z

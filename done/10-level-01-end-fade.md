## Your task: `level-01-end-fade`, Double Cross Part 1: the screen fade when the level ends

When Bond follows Mayhew down the last stairwell and the level ends, the port cuts straight to the next level: the fade-out the game plays at a level exit is missing. Find the game's level-end fade [M the exit trigger path Trigger_Update 0x1b1a78 case 10 / ResetMap_LevelToLoad, the screen fade it starts (Fade_* / the HUD fade pane), its length and colour] and port it for every level exit that uses it (01 is the reported one; check the other on-foot exits use the same path). Pin it with a test: on the 01 real-input walk's exit, the fade starts on the exit tick, runs the game's frame count, and the next level loads after it [M]. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-01-end-fade`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-29T16:36Z

---
## Report (done, 2026-10-29T18:03Z)


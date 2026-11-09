## Your task: `level-05-hurt-skip-2`, Bond's walked death in the 07000005 hurt volume is recorded as the game's

Branch from origin/s/level-05-hurt-skip, merge current origin/master, and read its docs/reports/level-05-hurt-skip.md plus the ignore message on its walked pin. The gatehouse leg walks clean, and the steering descent stops short of the valley. Take its next lever: the level doc's nav-lenient descent from the armour-pickup leg (59.65, 33.57, 86.11) to the valley floor (42.5, 8.9, 88.3), then walk Bond into the hurt volume and assert the -1000.0 death record the skeleton has in place. Helpers stay gated to 07000005. Real input only, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run walkreal_07000005_live, interactables_07000005_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-05-hurt-skip-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-09T08:05Z

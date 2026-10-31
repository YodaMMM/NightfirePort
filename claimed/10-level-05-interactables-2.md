## Your task: `level-05-interactables-2`, 05: every player-touchable kind behaves like the game

`interactables_07000005_live.rs` has 3 tests. Using `interactables_0700000b_live.rs` as the model (one live test per KIND of player-touchable placement — doors, switches/consoles, fuseboxes, lifts, pickups, breakables, plates, monitors, rigid bodies, wires/zip lines, grapple points, SimpleScripts the player triggers), run the census of 07000005 [D every placement kind], and for every kind a player can touch that is not yet covered, add the live test that drives it through the real path and asserts the game's result [M the class; D its params], with a sister that fails on the wrong result. Fix any port gap the tests expose, with evidence. Run interactables_07000005_live, walkreal_07000005_live and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-05-interactables-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-31T17:56Z

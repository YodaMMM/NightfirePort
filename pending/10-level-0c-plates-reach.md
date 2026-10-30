## Your task: `level-0c-plates-reach`, the last 0c stand-in: the plates reach

level-0c-vent-pulley-class (merged) made the display-eye leg real and kept one stand-in in `walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff`: the plates reach (the slab's south face z -155.77 closes every probed approach from the 2.37 deck; read that report's closure evidence). Find the game's own route to the plates [D every placement and collision face around the slab and the plates, including doors, SS movers, ladders, crouch gaps and the nav web; M any class you rely on], fix any port gap, and replace the teleport with a real-input leg. If the data truly shows no route, pin that with sister asserts and say exactly what a live check must answer. The walk must stay green. Quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-plates-reach`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


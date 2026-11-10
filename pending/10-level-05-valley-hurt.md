## Your task: `level-05-valley-hurt`, The Exchange's valley hurt planes hurt Bond when he walks onto them

Branch from origin/master. walkreal_07000005_live::the_valley_plane_hurts_bond_walked is ignored: the walked descent to hurt #3's plane stalls at the y-14.1 lip ((123.3,14.1,122.9), 6 legs identical) and the run+jump drive wedges at (105.6,25.1,114.6). Next lever from the note: the nav-lenient descent from the armour-pickup leg (59.65,33.57,86.11) to the valley floor (42.5,8.9,88.3), then north into hurt #4 (floor y 4.38). If a lip blocks a descent the game allows, that is a port collision bug: find it [M] and fix it. Assert the hurt values from the decompile. Un-ignore it. Run the whole walkreal_07000005_live binary and interactables_07000005_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-05-valley-hurt`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

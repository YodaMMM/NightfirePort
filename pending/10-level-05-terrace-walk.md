## Your task: `level-05-terrace-walk`, 05: reach the terrace spot on real input

`walkreal_07000005_live` still places Bond at `[22.0, 50.1, -68.0]` on the terrace (floor 49.001 [D probe_terrace05 col (21.8, -68.1)]) — read the comment above that write (around line 1365) for why it was a stand-in. Find the game's route to that terrace from where the test's walk stands [D collision/placements incl. doors, ladders, wires, grapple points; M the classes the route uses], walk it on real input and delete the write; if the data shows no route, pin that with a sister assert and name the live check that settles it. Run walkreal_07000005_live and interactables_07000005_live; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-05-terrace-walk`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


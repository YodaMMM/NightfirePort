## Your task: `level-16-walk-teleports`, 16: the walk's two placements to real input

`walkreal_07000016_live` still moves Bond with two `bond.pos` writes. For each: read its comment (why it was a stand-in), find the game's own route there [D collision/placements/nav; M the classes the route uses], walk it on real input, and delete the write; where the data shows no route, keep the write with a sister assert and name the live check that settles it. Do not touch the finale-pit question (the ch-0xf3 window) — that is separate. This level's archive is large: run single tests with --test-threads=1, and commit and push after each change so a reset loses nothing. Run walkreal_07000016_live and interactables_07000016_live; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-16-walk-teleports`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-31T15:45Z

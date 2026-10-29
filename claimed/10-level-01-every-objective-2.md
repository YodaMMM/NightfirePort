## Your task: `level-01-every-objective-2`, land the 07000001 objective walk

`origin/s/level-01-every-objective` is green but cannot merge: `walkreal_07000001_live` asserts `!channels.on(0x47)` ("07000001 never raises the obj0 completion channel - no disc writer on this level") as game truth. An assertion may not rest on a missing writer (the port may simply lack it). Merge `origin/master`, turn that assert into a `// [S] unverified` log kept out of pass/fail, keep every other assert (the cross-level fold on the 02 walk stays), and record what was searched for the 0x47 writer in the report. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-01-every-objective-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-29T10:13Z

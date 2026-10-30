## Your task: `level-0c-gate-band-crossing`, what carries Bond across x 55.5..67 on 0c

The 0c whole walk crosses the gate band x 55.5..67 on a stand-in: the port's disc collision is void there [D probe door4 pit map], yet the game's nav path17 links (49.69, 10.74, -207.29) <-> (63.06, 10.74, -207.54) across it [D]. Find what carries the player across in the game (a mover, a wire, a collision the port mis-reads) [D the placements/collision over the band; M the class code], fix the port side, and replace the stand-in with a real-input leg. The walk must stay green. Run the touched binaries and quote their totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-gate-band-crossing`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-30T06:27Z

## Your task: `level-0c-east-climb`, 0c: the game's way up to the east platform

`walkreal_0700000c_live` still places Bond at `[95.0, 11.3, -209.5]` for the east climb: staircase01's bottom floats 9.52 over the 6.35 under-stairs [D eastclimb3/4], the jump apex is 0.78, no grapple point serves it (the five `graple_pulley` points sit elsewhere) and the only ladders are #164 and #495 [D]. Find the game's route to that platform [D the placements, movers, script entities that pose collision (a lowered stair, a lift), the level's objective order — maybe the route reaches it from above; M any mover class], walk it on real input and delete the write; or, if the data shows no route, pin that with a sister assert and name the live check that settles it. Run walkreal_0700000c_live and quote the total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-east-climb`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-31T04:40Z

---
## Report (done, 2026-10-31T05:49Z)


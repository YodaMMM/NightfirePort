## Your task: `guard-leaf-press-2`, the 13-level leaf press without breaking the Double Cross exit

Branch from origin/s/guard-leaf-press-07 and merge origin/master. Merged alone onto master it turns fix_mission_live::the_07000001_exit_works_end_to_end red (master passes). Read the message: likely the leaf press holds a guard or Mayhew at the courtyard shoji or a door on the exit route. Fix it the game's way [M Drone_CollisionHandler 0x140968:204-208] so the exit works again, then run every walkreal_*_live binary one level per tool call and the full suite.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/20-guard-leaf-press-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox7 at 2026-11-12T02:59Z

---
## Report (blocked, 2026-11-12T04:02Z)


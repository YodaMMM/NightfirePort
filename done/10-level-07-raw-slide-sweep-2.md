## Your task: `level-07-raw-slide-sweep-2`, the doorway raw-slide fix lands without breaking any level's walk

Branch from origin/s/level-07-raw-slide-sweep, merge current origin/master, and read its docs/reports/level-07-raw-slide-sweep.md. The sweep confirmed the regression the doorway fix warned about: 01 and 02 die on the fix's base and pass on master (the shoji crossing), with three confirmed regressions in all. Take "What's left" in order. Use the two-flip isolation the report describes to pin which arm of the fix's lineage causes each regression, then fix each one the way its diagnosis points, never bringing the probe back (the game's rule is [M 0x139268:70-93]). Then the same isolation for the 0a runner-chase family, the 03 chase-kill and the 08 Agent deaths. Re-run the three binaries that ran out of memory, one per tool call, and the remaining level walks (11 route, 12-16, 1b, 41, 4a and 4a satchel). Real input only, never lower damage or health. Run every walkreal_*_live one level per tool call, npc_doors_live, fix_guard_misc_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-07-raw-slide-sweep-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-08T16:28Z

---
## Report (done, 2026-11-08T17:24Z)

Closed by the driver: the report is on `s/level-07-raw-slide-sweep-2` (docs/reports/level-07-raw-slide-sweep-2.md); the worker didn't run finish.sh.

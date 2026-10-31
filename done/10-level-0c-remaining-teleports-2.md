## Your task: `level-0c-remaining-teleports-2`, 0c: the walk's last placements

`walkreal_0700000c_live` and `walkreal_0700000c_eastroute_live` still carry `bond.pos` writes (the trench exit, the east climb, the ladder495 measurement spot, the eastroute start/jump spots). Several are now settled facts — the trench is the vent shaft's one-way drop, the east band deck is a collision island [D], the ladder495 spot is a measurement spot. For each remaining write: if it is a route stand-in, find the game's route and walk it on real input; if it is a settled no-route or a measurement spot, make sure the reason and a sister assert sit next to it. Delete every write that can go. Run walkreal_0700000c_live, walkreal_0700000c_eastroute_live and interactables_0700000c_live; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-remaining-teleports-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-31T16:01Z

---
## Report (done, 2026-10-31T18:28Z)

Closed by the driver: the report is on `s/level-0c-remaining-teleports-2` (docs/reports/level-0c-remaining-teleports-2.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-10-31T18:35Z)


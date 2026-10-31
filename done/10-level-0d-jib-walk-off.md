## Your task: `level-0d-jib-walk-off`, 0d: the crane deck to the yard on real input, the last legs

level-0d-jib-rim-gap (merged; read its report and `probe_crane17/18/19`) mapped the deck -> yard route and found the port's jump and the script-entity collision posing faithful [M Player_HandleJump 0x1916b0; Player_SSWalk 0x192310:69]. A mid-swing walk-off reached the walkway strip's r band but landed ~0.13 m outside its width. Finish the chain on real input: (1) the walk-off — a deck-edge bearing whose lateral offset lies inside the strip, or the roof-side edge (18.58); (2) the dock walk + E #460 in-route; (3) the beam boarding via the north steps and the roof/catwalk onto SP #115's westward pass, the ride west and the drop into the yard. Replace the remaining teleport in `walkreal_0700000d_live::walk_arrival_to_the_c5_door`. If a leg needs a port fix, cite the class [M]. Every test in the file green; run walkreal_0700000d_live and interactables_0700000d_live and quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-jib-walk-off`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-31T05:21Z

---
## Report (done, 2026-10-31T06:33Z)

Closed by the driver: the report is on `s/level-0d-jib-walk-off` (docs/reports/level-0d-jib-walk-off.md); the worker didn't run finish.sh.

## Your task: `level-0d-beam-boarding`, 0d: board the shipping beam on real input

level-0d-last-two-standins (merged; read its report) walked the catwalk exit via the west-end ladder and left ONE narrowed stand-in in `walkreal_0700000d_live::walk_arrival_to_the_c5_door`: the beam boarding + the west ride on SP #115's shuttle (137.54 <-> 177.22 over 3000 ticks [D]) and the drop into the yard, with the boarding surfaces pinned and the access question reduced to one live check. Settle the access from the data where you can [D the beam's script-entity collision at each tick of the shuttle, the roof/catwalk edges it passes; M Control_InheritVelocity 0x133e88 / Player_FeetOnPoint 0x194188 for the carry], walk the boarding and the ride on real input, and delete the stand-in; if only a live run can settle it, pin the exact question with a sister assert. Commit and push after each finding. Run walkreal_0700000d_live and interactables_0700000d_live; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-beam-boarding`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-31T14:59Z

---
## Report (done, 2026-10-31T16:21Z)


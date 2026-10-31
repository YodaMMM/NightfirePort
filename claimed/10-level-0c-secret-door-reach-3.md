## Your task: `level-0c-secret-door-reach-3`, 0c: the real way to the secret door's plates

Continue `origin/s/level-0c-secret-door-reach-2` (merge `origin/master` first; read its report in `blocked/10-level-0c-secret-door-reach-2.md`). Fresh-boot live runs of the original game settled its three live checks [L 2026-10-31]: (1) walking west from the warehouse ground is stopped at x 56.3-56.75 for z -152..-166 (the apron slab) but opens at z -146/-141 (Bond reaches x 50.4); (2) from there, walking SOUTH into the corridor (x 53.0-55.6) is stopped at z -150.23 by a BARREL ROW in every lane, and running jumps do not clear it — so the corridor's north end is CLOSED in the game; the static-collision flood that found "one component" did not include the barrels; (3) the ST-1 walkway's south end (13.5, 15.7, -152) has a railing: Bond stops at z -153.65, no drop. So find the route the game really uses to the plates #558/#559: the barrels' kind and whether they can be destroyed or moved (a destroyable/rigid-body/0x3B class), a drop from the mezzanine deck (5.44) into the corridor, the annex vent #180/#182/#184 (ch 0x54), or a door [D placements + collision incl. dynamic objects; M the classes]. Make the port's barrels block exactly like the game's (a test pinning the stop at z -150.2 from the north), walk the real route on real input, open the door through the plates, and delete the `[plate2 + 0.6]` placement in `walkreal_0700000c_live`. Run walkreal_0700000c_live and interactables_0700000c_live; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-secret-door-reach-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-31T12:31Z

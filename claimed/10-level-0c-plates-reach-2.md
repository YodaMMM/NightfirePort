## Your task: `level-0c-plates-reach-2`, the pocket-to-yard link and the last 0c teleport

level-0c-plates-reach (merged) walked the route display ground -> 2.18 lane -> storage room -> the 2.37 pocket (`probe0c_plates_route`) and mapped the yard side to the plates; ONE link is unmapped: the pocket -> yard descent (the slab face z -155.77 spans x -4.5..15.25; the pocket's west edge x -1.8 hangs over void; the z -158.5 wall's sloping top 4.55..6.69). Work its "What's left" candidates (a) wider coll-scan west of x -4.5 / south of z -155.7, (b) the sloping wall top as a walk, (c) what lies above the yard's north-west arm [D]. If a route exists, rewrite leg 4 of `walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff` on real input with no teleport; if none, pin that with sister asserts. The walk must stay green. Quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-plates-reach-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-30T12:14Z

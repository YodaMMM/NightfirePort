## Your task: `level-06-castle-walks`, 07000006: the castle interior walks, un-ignored

`walkreal_07000006_live::walk_the_castle_interior_to_the_upper_gallery` and `stopping_before_the_tunnel_never_ends_the_castle` are still ignored. Their old blocker (the fire fault: one round per 12 presses, the view turning on the press tick) does not reproduce on master — `fix_06_fire_live` pins every press firing and no body turn.

1. Run both walks un-ignored on master. Fix what stops them with real input and the game's rules [M] — the courtyard fight with the alarm's reinforcements, the doors, the route to the upper gallery.
2. Un-ignore both when they walk; the sister (stopping before the tunnel never ends the level) must still be able to fail.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-06-castle-walks`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-10-25T01:52Z

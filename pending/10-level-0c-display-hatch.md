## Your task: `level-0c-display-hatch`, 0700000c: the vent route into the display case

Two `fix_0c_hatch_live` tests are ignored:
- `bond_crouches_through_the_open_vent_to_the_display_ground` — leg 1, the ascent to the duct top, is unidentified: the game's climb is ladder-only [M 0x196118] and the vent has no ladder [D];
- `from_the_duct_top_the_open_hole_carries_bond_down` — legs 2-3 work (the open hole drops the crouched capsule onto the inner ledge 14.72); leg 4, the descent into the display case (floor ~7.3 at (5.4, -113) under the 11.27 mid floor), is not mapped.

1. Find how the player reaches the duct top in the game [D + M — nearby crates/ledges/jumps, a script or a lift, the Q-Claw grapple points on 0c] and walk it with real input.
2. Map and walk leg 4 into the display case [D the floors]; fix any port divergence [M].
3. Un-ignore both tests as they go green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-display-hatch`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

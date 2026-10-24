## Your task: `level-14-trench-landing`, 07000014: the drop's ghost landing, the east corridor, the mezzanine route

Master has the game's drone-vs-player push (`Collide_Jointy` 0x1ec180's 0x800 arm). The real-input east route (`walkreal_07000014_live::the_east_duct_route_reaches_the_stairwell_slot`, ignored) now stops at the drop: Bond rests on nothing at y 2.97 over the trench — `capsule_push` walkable with push ~0, no floor triangle within 3 m below, no dynamic triangle in the stall box. Read `docs/reports/level-14-sleeper-push.md` on `origin/s/level-14-sleeper-push`.

1. Dump the triangle owners at the stall pin. Suspects: the flipped-open lid's final pose (its anim ends at a 180-degree flip north of the hinge) or the script's second entity 0x02000751 (obj_type 25, created at key 61) standing in the slot. Find what the port collides with that the game does not (or the reverse) [M] and fix it.
2. The corridor east (x 41 -> 49 at z 62.9), the x ~49 south run to the mouths (z 42.16 — the same mask-4 lids #1/#3, E to open) and the mezzanine. Decide [D + M] whether the grapple points #626/#657 at (45.30 / 40.78, 4.79, 42.86) are the player's route up rather than the stairwell slot.
3. Un-ignore `the_east_duct_route_reaches_the_stairwell_slot` and `walk_the_infiltration_to_the_delta_exit` as they go green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-trench-landing`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.



---
claimed-by: sandbox3 at 2026-10-24T20:10Z

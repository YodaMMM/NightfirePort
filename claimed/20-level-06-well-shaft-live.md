## Your task: `level-06-well-shaft-live`, 06: is the well shaft open below the hanging body?

`interact_07000006_live::the_well_shaft_is_open_and_the_hurt_volume_is_game_inert` is ignored: since anonymous rigid bodies register with their shape key [M RB_Create 0x1d6bf0:81, Collide_Pick 0x1eb320:314-329], body #65 (about 0.2 wide, 0.92 tall) at the centre of the well mouth is solid and Bond lands on it at y 39.96; and 0.7 off centre Bond falls through the shaft floor (to y -356) although the floor probe finds it about 6.2 below. Settle both from data: [D] #65's glist collision and the shaft floor's collision triangles (one-sided? flags 0/99?), [M] whether a 0xD8 body of this type collides with the player capsule (its type/flags, `Player_Collision` 0x194df0's query mask vs obj+0xF0). Fix what the port gets wrong (the fall-through especially), re-pin the well test honestly with citations and un-ignore it. Run every *07000006* binary, rigid_body_* and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-06-well-shaft-live`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-01T07:21Z

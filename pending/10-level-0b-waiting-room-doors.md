## Your task: `level-0b-waiting-room-doors`, 0b: walking through the open waiting-room doors

level-0b-teleports-to-walks (merged; read its report and `probe_0b_execfloor_nav`) kept one movement stand-in in `walkreal_0700000b_live`: the waiting-room exit (lift landing -> door #3 (13.71, 46.78, 19.44) -> the east room -> door #0 (13.82, 46.78, 7.09) -> the ring's east corridor). The doors open (state 0x2 -> 0x4) but the walker wedges 2.5-4.3 m short of #3 and still blocks at #0's doorway. Movement collision is state-blind; passability comes from the open leaf's pose [M Door_Interp 0x135738:33-61 writes obj+0x90; control_movement_object_handler 0x1336f8:147-236; Collide_PickObj 0x1eae80]. Find which end value (0.0 / 1.0) these doors open to and what pose the port gives the open leaf vs the game's law [D the door params/anim; M Door_Interp], fix the port's open-leaf pose or the doorway cels, and walk the transit on real input (delete the write). Run walkreal_0700000b_live, interactables_0700000b_live and every *door* binary; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0b-waiting-room-doors`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


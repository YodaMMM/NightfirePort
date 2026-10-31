## Your task: `level-0b-waiting-room-doors-2`, 0b: why the waiting-room door swing reverses

Continue `origin/s/level-0b-waiting-room-doors` (merge `origin/master` first; read its report and `probe_0b_doors`). Its trace is good evidence: Bond standing ~1.2 m from the hinge is a permanent contact, and `Door_ReactToHit`'s anti-crush [M 0x135980:85-97] reverses the swing every ~3 ticks, so the leaf never clears. But its fix (the door's radius alone in `Doors::collide_step`) is the WRONG arm: `Collide_PickObj` 0x1eae80 picks that radius only for hittest mode 0x200/0x201 (:91-107); the player's mode (BL+0x7D4 = hittest+0x94) is 0x800 on every `Player_Collision` 0x194df0 arm (:205, 229, 257, 282, 315, 363, 381 — 0 only at :308), and the 0x800 arm (:152-168) is `1.5*r_door + r_pick` — the code master already has. Revert that change and find the real difference: the r_pick value the port passes (hittest+0x8C for each Player_Collision arm), the door's live sphere (obj+0x80 centre / +0x8C radius after Door_Interp), the anti-crush condition and its local.z sign [M 0x135980], or where the game stands Bond after the E [M Door_Activate 0x135be0]. Fix it with that evidence, then walk the transit (resume the walked rewrite in f3873e62) and delete the stand-in. Run walkreal_0700000b_live, interactables_0700000b_live and every *door* binary; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0b-waiting-room-doors-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


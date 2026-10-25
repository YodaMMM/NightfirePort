## Your task: `level-07-rook-doorway`, 07000007: Rook freezes in the west doorway of the party level

In real play on the party level 07000007, Rook walks his patrol and stops dead in a doorway (Bond at (76.13, 44.13, -51.97), frame 1086, facing him). This is drone 22 (vt 0x10, ch 23, team 3, skin 0x5000026, walking "Rook Patrol Path_3"), which freezes at (79.53, 44.0, -50.32) inside the west doorway (door at (80.0, 43.0, -49.9)). (Earlier Rook tasks looked at 07000006 by mistake.)

Round 3 of the guest-collision work (`origin/s/level-07-guest-collision-3`, docs/reports/level-07-guest-collision-3.md) found the cause: the game moves an UNDRAWN walking drone by a raw position add with no collision test [M NDrone2_DoAnimation 0x149f88:11-18, Drone_Control 0x139268:75-96], while the port routes that slide through `move_by`'s probe, whose rays graze the doorway throat and reject every step. Round 4 (`origin/s/level-07-guest-collision-4`) found the raw add alone lets an undrawn route-stalled drone cross the closed 01 shoji (npc_doors_live), because the game also clamps the slide by the boundary bits [M 0x139268:76-83, DroneMove_SetBoundryFlags 0x14f140] and its route stall swaps the walk danim for the stall anim.

1. Land ONLY the undrawn-slide part (not the player push), with the game's boundary clamp and stall-anim swap so both the 07 doorway and the 01 shoji are right [M].
2. Real-input test: Rook completes Path_3 through the west doorway; npc_doors_live, walkreal_07000007_live and the 01 walks stay green. Run the FULL suite on the merged tree; report the total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-07-rook-doorway`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox3 at 2026-10-25T17:21Z

---
## Report (done, 2026-10-25T18:46Z)


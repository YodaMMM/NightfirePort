## Your task: `level-0d-c5-door-laser`, Chain Reaction's C5 door is cut down by the deck laser like the game

On 07000d (Chain Reaction Part 2) the C5 doors are Destroy objects (class 0x21): placements #93 bigdoor01 / #96 bigdoor02, p0 ch 0x1f, p1 trigger 0x1f, p2 type 5/6, p3 health 1000/10000, p4 mask 0x10, p5 blast 10, p6 delay 0 [D]. Master currently smashes C5 on the FIRST beam contact through Explode_Propagate case 0x21 - that is wrong: case 0x21 smashes only when ext+0x14 & 0x20 == 0 [M Explode_Propagate 0x17c518 case 0x21], and Collide_FilterBullets 0x1ece30 drops every hit whose flags meet ext+0x14 = ~mask (0xFFEF): player guns (flags 0x08) do nothing, the mounted guns 0x61/0x67/0x70 (flags 0x10) pass.

The game's real sequence, measured live [L PCSX2 2026-10-27, Bond on the deck laser mount at (137.27, 10.5, -326.0)]:
- With the beam on bigdoor02's face its health (ext+4, f32) drains ~50 per frame of contact (10000 -> 7100 -> 4050 over two 60-frame bursts); off-face frames do nothing; ~200 contact frames kill it. Find the beam's damage path [M GunImp_Update 0x1320d8's hit arm -> the collision handler -> Destroy_CollisionHandler] and port it exactly.
- While hit, the door's colour bytes obj+0x104/0x105 drop to 0x00 (rgba 00ff0000) and recover ~3 per frame once the beam leaves (00ff3b3b, 00ff7a7a, 00ffbbbb, 00ffffff) [M Destroy_CollisionHandler + Destroy_Update for the exact step]. Expose that colour on the object for the draw (the main agent renders it).
- At hp <= 0 (measured -50): Destroy_Smash 0x12bb60 - latch ext+0x17 = 1, class byte 0x21 -> 0x01, glist swaps to the destroyed glist (glist + 0x50 in RAM: the object's LOD/destroyed entry), light removal, the 0x6000052 explosion x3, channel 0x1f written - and bigdoor01 smashes on the channel in the SAME frame at full health 1000 [M Destroy_Update trigger read]. ~24 frames later a fireball fills the doorway; ~190 frames later both leaves are gone and the doorway is open.
Port all of it: remove the instant smash (honour the 0x20 gate), the bullet mask filter, the beam damage at the game's rate, the hit-colour bytes, the smash with the destroyed glist + the channel chain + the objective it advances. Tests on real input from the mount: player-gun fire does nothing; the beam drains health at the measured rate; the smash fires at <= 0 with both doors and ch 0x1f; the objective advances. Correct the merged level-0d-gun-door-objective test that pins the one-contact smash (cite this brief's [L]). Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-c5-door-laser`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-28T05:24Z

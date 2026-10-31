
### level-0b-waiting-room-doors (start, 02:12 UTC, 2026-10-31)

Task: walkreal_0700000b_live keeps ONE movement stand-in — the waiting-room exit (the lift landing
→ door #3 (13.71, 46.78, 19.44) → the east room → door #0 (13.82, 46.78, 7.09) → the ring's east
corridor). The doors open (state 0x2 → 0x4) but the walker wedges 2.5-4.3 m short of #3 and still
blocks at #0's doorway. Movement collision is state-blind; passability comes from the open leaf's
pose [M Door_Interp 0x135738:33-61 writes obj+0x90; control_movement_object_handler 0x1336f8;
Collide_PickObj 0x1eae80]. Find which end value (0.0/1.0) these doors open to and what pose the
port gives the open leaf vs the game's law [D door params/anim; M Door_Interp], fix the port's
open-leaf pose or the doorway cels, walk the transit on real input (delete the write). Run
walkreal_0700000b_live, interactables_0700000b_live and every *door* binary; quote totals.

### level-0b-waiting-room-doors (finish, 03:55 UTC, 2026-10-31)

The stand-in's cause found and fixed: the port's door pick used `1.5·r_door + r_pick` where the
game's law [M Collide_PickObj 0x1eae80:100-110] is the door's own obj+0x8C radius ALONE — the
phantom E-activator contact drove Door_ReactToHit's anti-crush [M 0x135980:85-97] to reverse
every E-open swing ~every 3 ticks (NF_DOORTRACE caught the oscillation), so the open leaf never
cleared the doorway. The doors' law established: closed = the middle key, the open ends = keys
0/2, the end value from the approach side [M Door_Activate 0x135be0:136-146], +0x5C derived by
Door_SetupSwing [M 0x135440:65-80, Mat_GetDir 0x1e3398 = row 2, Mat_GetNorm 0x1e33d8 = row 0] —
the port's pose law matches. Measured ends: door #3 → pos01 1.000, door #0 → 0.000 (state 4).
probe_0b_doors committed. The walked-transit rewrite is mid-flight (commit f3873e62 on this
branch); the test head is the green form. Verified: walkreal_0700000b_live 3/0,
interactables_0700000b_live 9/0, all 15 *door* binaries 35 passed / 0 failed / 2 ignored.

## level-0b-waiting-room-doors-2 (branch s/level-0b-waiting-room-doors-2)

- start (from s/level-0b-waiting-room-doors 5aea1206; merge origin/master 20624c8c clean). Task:
  the prior pick fix (the door's radius alone in `Doors::collide_step`) is the WRONG arm —
  `Collide_PickObj` 0x1eae80 picks that radius only for hittest mode 0x200/0x201 (:91-107); the
  player's mode (BL+0x7D4 = hittest+0x94) is 0x800 on every `Player_Collision` arm (:205, 229,
  257, 282, 315, 363, 381 — 0 only at :308) and the 0x800 arm (:152-168) is `1.5*r_door + r_pick`
  — the code master already has. Revert it; find the real difference (the r_pick value the port
  passes (hittest+0x8C per arm), the door's live sphere (obj+0x80/+0x8C after Door_Interp), the
  anti-crush condition and its local.z sign [M 0x135980], or the post-E stand [M Door_Activate
  0x135be0]); fix with evidence; resume the walked rewrite (f3873e62) and delete the stand-in.
  Run walkreal_0700000b_live, interactables_0700000b_live and every *door* binary.
- the pick REVERTED to the game's 0x800 arm `1.5·r_door + r_pick` [M Collide_PickObj 0x1eae80:152-168 —
  the prior branch's radius-alone law is the 0x200/0x201 arm :91-107; the player's mode word
  BL+0x7D4 = hittest+0x94 = 0x800 on every Player_Collision arm and r_pick = hittest+0x8C = 0.55
  walk / 0.275 climb]. The real difference found: the port's law WAS faithful — the wedge is
  `Door_ReactToHit`'s anti-crush [M 0x135980:85-97] hitting its geometry: Bond's live-frame lz
  crosses zero mid-swing (the NF_DOORTRACE capture: lz +0.353 → −0.028 as pos01 0.48 → 0.34,
  swing −1), so the E-activated away-swing reverses at the crossing and the closing arm reverses
  back — a 3-tick limit cycle (0.3448 ↔ 0.3966) that holds the leaf near-closed while he stands
  at the E spot. The game's flow never waits it out: the player walks THROUGH the swinging
  doorway. probe_0b_doors now proves both crossings walk (+70/+73 ticks; both leaves complete to
  1.000/0.000 once Bond is through). The walked transit is IN walkreal_0700000b_live: door #3
  through +62 ticks, the east-room nav lane, door #0 through +67 ticks, ring n11 — the big
  stand-in is GONE; kept: the ~6 m dock→east-approach hop (the waiting room's floor maze: the
  dock band is an island — the pit x 3-8, the x 2-3.5 hole, the dome hole; the main floor's
  x 4.5-6.5 barrier — the floor grid in probe_0b_doors carries it). walkreal_0700000b 3/3.

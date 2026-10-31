
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

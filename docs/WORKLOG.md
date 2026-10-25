## level-07-rook-doorway (2026-10-25, sandbox expiring ~19:26 UTC)
Branch s/level-07-rook-doorway off origin/master (62f70bcc). Task: 07000007 party level — Rook
(drone 22, vt 0x10, ch 23, team 3, skin 0x5000026, "Rook Patrol Path_3") freezes at
(79.53, 44.0, -50.32) inside the west doorway (door at (80.0, 43.0, -49.9)) [driver probe,
frame 1086, Bond at (76.13, 44.13, -51.97)]. Rounds 3-4 of guest-collision found: the game moves
an UNDRAWN walking drone by a raw position add with no collision test [M NDrone2_DoAnimation
0x149f88:11-18, Drone_Control 0x139268:75-96] while the port routes the slide through move_by's
probe (rays graze the doorway throat, every step rejected); the raw add alone lets an undrawn
route-stalled drone cross the closed 01 shoji, because the game also clamps the slide by the
boundary bits [M 0x139268:76-83, DroneMove_SetBoundryFlags 0x14f140] and its route stall swaps
the walk danim for the stall anim. Plan: land ONLY the undrawn-slide (not the player push) with
the game's boundary clamp + stall-anim swap; real-input test — Rook completes Path_3 through the
west doorway; npc_doors_live, walkreal_07000007_live, the 01 walks stay green; full suite on the
merged tree, total line in the report.

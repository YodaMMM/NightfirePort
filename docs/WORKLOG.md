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

Outcome (same session, 19:1x UTC): landed the undrawn slide as the game's raw add [M
Drone_Control 0x139268:75-96] with the game's boundary clamp [M :76-83] and the SetBoundryFlags
probes [M 0x14f140] ported live in the control tick behind the +0x23 flag [M 0x1490f0:202-206].
KEY FINDING: the fold doc's "the +0x4F8 bit-1 setter is unaccounted" was wrong —
DefaultInit 0x14b300:60 arms bit 1 with the composite init OR 0x801e2 (the only clearer is
AstronautLaunch 0x16dd28:19) — the clamp is LIVE on the shipped paths; doc corrected. Probe
evidence (scratch example, untracked): at the 01 shoji the faithful forward probe (feet +0.5
local, BoundsTest only — no collision LOS per MoveTest 0x155330:95-108) hits the 0x800 door
wall at d 0.416 → the clamp stops Mayhew exactly where the game does; at Rook's 07 freeze spot
the walking-direction probe passes. Repro cycle: raw add alone turned npc_doors_live's
the_01_escort_driven_through_the_shoji red (Mayhew crossed the CLOSED shoji, exited 0x63 at
(6.7,12.4)); with the live clamp the whole binary is green again 6 passed/1 ignored. Green:
npc_doors_live, walkreal_07000007_live 2 passed/1 ignored (the new Path_3 leg), nf-game builds.
Red/ignored left-work: (1) the new doorway-leg test ignored — under the live clamp walker22's
start pose (80.15,−50.46) re-freezes: the port's ONE-CEL boundary flattening probes wall
#684/#685 across the west doorway where the game's straddled-cel BLINK walk [M 0x157d80:113-158]
would not (round-3's raw-add-only tree passed the leg) — next round ports the per-cel filter;
(2) walkreal_07000001_live's contact walk fails at the mission-monitor assert (states01[0]==4)
after progressing past the ch-0x99 guard kills — the guards' game-correct raw-add positions
shift the harness's flanking lines into the level's known pockets; the harness gained round-3's
pocket back-off (walker.rs, stuck 3/4 → 60-tick reverse). No full-suite total line this session
(the sandbox kills background jobs between tool calls; the foreground slices covered the touched
binaries) — the next round runs tools/test/suite.py on the merged tree. Live checks: gdbwatch
drone+0x4f8 bit 1 at 07000007 boot (expect set) and 0x320 bits for an undrawn walker at the west
doorway (expect the walking-direction bit CLEAR — settles the per-cel BLINK question).

## level-07-rook-doorway-2 (2026-10-25, sandbox expiring ~21:44 UTC)
Branch s/level-07-rook-doorway-2 off origin/s/level-07-rook-doorway + origin/master. Task: port
the game's per-cel bounds test exactly [M AINetwork_BoundsTest 0x155330, the straddled-cel BLINK
walk M 0x157d80:113-158, the cel the drone stands in] so the clamp probe sees only the walls the
game's does; un-ignore the_mission_walker_completes_his_doorway_leg; Rook (drone 22, Path_3)
crosses the west doorway (79.53, 44.0, -50.32) in real play. npc_doors_live and the 01/07 walks
stay green; full suite on the merged tree, total line in the report.

Work started (19:4x UTC): branch created from rain-5's merge base, origin/master merged, pushed
before any code change. Setup reused: work/ slim tree and Nightfire-onfoot.iso already present.

Outcome (level-07-rook-doorway-2, 20:4x UTC): the per-cel port landed — bounds_test now builds the
game's exact shape: find_cel(a) → Collide_StraddleCels (the world's straddle walk, the +1.0 y lift,
the 0x201 ray portal test) → the to-cel membership gate [M 0x157d80:106-127, 0x1eac98] → the walk
over ONLY the walls whose post cels (build_FindCel of the post, [M 0x1d2f18:30-34]) are straddled;
Wall carries ca/cb, the flat walk stays as the no-scope stand-in for the tests' unscoped worlds.
KEY FINDING (the real freeze cause): the previous round's fold-doc correction armed "+0x4F8 bit 1"
but both push-move gates read BIT 0 [M 0x14f140:47 `(x^1)&1==0`, 0x139268:80 `uVar2&1`], and
DefaultInit 0x14b300:60's composite `(x & 0xffea05e2) | 0x801e2` CLEARS bit 0 (the ~mask carries
it) and never re-arms it — a full-decompile scan found no `| 1` writer into +0x4f8. The game's
undrawn raw slide is UNCLAMPED for default-init drones; the port's push_move true (off the bit-1
misread) was the freeze: Rook's forward probe crossed the east jamb wall #685 (cel 8, in the
straddle set) at 0.006 into the probe and the clamp zeroed the 0.02 slide every tick (boundary
0x12, travelled 0.00). push_move=false; the doorway-leg test un-ignored and GREEN (Rook crosses);
the 07 walks 3/3 green (the full party walk + the zone test + the leg), the 01 walks 3/3 green
(including the contact walk), nav unit tests 8/8. RED left: npc_doors_live's
the_01_escort_driven_through_the_shoji — the driven 0x63 Mayhew crosses the CLOSED never-activated
shoji (closest 1.33): the clamp was the substituted stopper; the game's real one is not the
OptimiseRoute trims (end-only [M 0x156760:59-116]) nor the Handler post probe (r 0.2 [M
0x3e4ccccd]) — by elimination the drone-body vs closed-leaf world push (NDrone2_Collision
0x140e90; the port's drone consults only the AI boundary push_out, which skips 0x800 walls, and
the doors ARE in set_dynamic) — next round traces the port's drone-vs-dynamic-leaf push on the
shoji approach. Scratch probes (untracked): nf-view/examples/scratch_celprobe07.rs,
scratch_rooktick.rs. No full-suite total line yet (background jobs die between tool calls here);
foreground slices: nf-game lib nav, walkreal_07000001/07, npc_doors_live.

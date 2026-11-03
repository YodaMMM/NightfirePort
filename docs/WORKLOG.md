
## level-12-alarm-guard-walls (2026-11-03)

Task: on 07000012, after the alarm, guards running toward Bond stop and press against empty
space near (22.97, 1.06, 36.21) (frame 1391 of a real play; nothing drawn there). Reproduce
on real input: walk the walkreal_07000012 route, trip the alarm the way a player does,
record every drone whose position stops changing while chasing/attacking; for each wedge
find the blocker (static collision [D coll-scan], door/blocker volume, nav link, port bot
push) and compare with the game's movement/collision path [M NDrone2_DoCollision 0x149ec8,
Collide_Jointy 0x1ec180, nav/obstacle routines]; fix the port where it blocks where the
game does not; pin a live test (after the alarm every chasing guard reaches Bond's area
within bounded ticks, none sits >N ticks against a spot with no [D] geometry). Run
walkreal_07000012_live, interactables_07000012_live and nf-game.

Branch cut from origin/master c0450ef6. Sandbox reset at start: no repo/work/ISO, setup
re-run in foreground.

Session result (01:45 UTC): the wedge named to geometry and the game's alarm path read; no fix
landed — nothing proven broken yet. Evidence: (1) [D scans, committed probe walkreal_probe12f]
(22.97, 1.06, 36.21) is INSIDE the real cafeteria room — static floor y 0.0, ceiling ~3.13,
nav nodes path#8[9] (22.06,0.45,33.34) / [180] (25.33,0.52,35.80) / [181] (19.35,0.52,34.18)
within 3 u; the BLINK boundary walls #0-#5 bound it: #1 (37.23,0.42,36.66)->(22.47,0.42,36.64)
h 2.53, #2 ->(20.00,36.14), #3 ->(17.99,34.82), #4 the z 32.0 south edge; the spot is 0.43 u
south of the wall#1/#2 corner. BLINKs are AI boundaries, not drawn geometry — "nothing is
drawn there" is what a BLINK corner looks like. The room's only opening to the offices is the
corridor mouth x 17-19 z 31-33.5; a static wall stands at x ~18.8-19.8 z 36.2. (2) [M] the
game's alarm path: Drone_InitComms 0x139520:255-259 broadcasts Drone_Message(0,0x1e) every 30
frames while switch_channels[0x96]; Drone_Message 0x13a678:21-23 (obj 0) -> SM_BroadcastMsg ->
SM_RouteMsg 0x172128:106-142 delivers to every drone whose class obj+0x10c is not in
{1,3,0xf,0x47..0x48} — no enabled gate; NDrone2_DSTATE_WaitSwitch 0x15b960 handles msgs
1/3/0xe only — msg 0x1e hits `return 0`: a WAIT_SWITCH guard IGNORES the alarm (so the port's
enabled=false filter at drone.rs:18101 is behaviour-faithful for the alarm path; the port's
WAIT_SWITCH arms drone.rs:7328-7330 match). (3) [M 0x140e90:169-174] the game's own
NDrone2_Collision boundary-pushes every bounded guard per tick via
AINetwork_GetBoundsPushVectorForSphere(0.4) — the port mirrors it (nav.rs push_out
[M 0x157bd0], drone.rs:19789-19796). (4) Headless repros (probe, log-only): alarm-charge along
the route; Bond in the cafeteria through the corridor touch #30 (ch 0x5 fires, enabling the
west-wing pair plc 21/22 [D params]); whole-roster stress (every WAIT_SWITCH guard enabled via
the game's msg 0x0e then alarmed). Result in every run: the corridor pair (#8 plc 12, #9 plc
13) and the lobby pair (#28/#29 plc 691/692) advance only to the corridor mouth (z 24-31) and
park in stand/crouch/cover-fire (one emits the cover goal (13.06, 30.65)); the west-wing pair
patrols its wing; NO guard enters the cafeteria; NO guard wedges at (22.97, 36.21). The
reported press therefore needs guards INSIDE the cafeteria, which the port never produces in
these repros — the open question is the game's advance law (COMBAT_OUT_OF_RANGE approach /
COMBAT_NO_SIGHT search) vs the port's fire-state park, and whether the game's guards at that
corner are themselves boundary-pushed [M 0x140e90:169-174] (the measured 0.43 u matches the
port's push_out r 0.4 + wall slack). Live checks filed in the report. Ran:
walkreal_07000012_live 2/2, interactables_07000012_live 12/12, nf-game lib 655/655 (+5
ignored); the background suite.py relaunch is in flight (background jobs die between tool
calls in this sandbox — the four-crate run may not fit the window).


## level-03-real-health-servant (started 2026-11-02 14:59 UTC)

Branch cut from origin/master 84e64088. Task: un-ignore
walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_real_damage — on the
real-damage run the route's "the servant carrying 0x4f [D p6]" lookup (~line 231) panics (not a
Bond death; 98.5 hp at tick 22200), while the top-up walk passes the same assertion. Find why the
runs differ (return fire moving/killing the servant, the kill order, the damage log's return fire)
and whether the game would let that happen; fix the walk or the port (cited); real input only;
then walkreal_07000003_live, every real-damage walk, full suite.

## level-03-real-health-servant — the diagnosis + the fixes (2026-11-02 16:18 UTC)

Real-damage differential: the run's courtyard approach crosses the EAST 0x21 twin at tick 7165
[the port's ch read] — servant #389 enables and binds a courtyard captor [M 0x144030] — and the
fight's kills reach that captor: Hostage case 3's captor-death check [M 0x15cb78:24-36] fires
HostageSaved → switch_channels[+0x135] [M 0x1441c0:11-18], PreDroneControl consumes the byte
[M 0x148db0:51-90] — the game's own designed save, earlier than the scripted leg. The old test
sampled the servants AFTER the fights and the "the servant carrying 0x4f [D p6]" lookup read the
consumed byte (0x0) — the game would let this happen; the test's assumption was wrong. Fixes:
(1) the sample moved before the fights (the p6 bytes are the placements' own words [D p6]); the
rescue loop's live channels.on() check already skips the captor shot for an early rescue.
(2) The decompile also exposed a real port leak the differential's Drone log caught (the saved
servant knocked out of HostageHide through an Attack re-route into CivilianHiding): the
scream/shout receiver block ran for WAIT_SWITCH and the hostage family, whose game handlers have
no 0x11/0x12 case [M 0x15b960 / 0x15cb78 / 0x15ccf0 / 0x15ce60 / 0x15cfe0] and neither does the
unhandled→GLOBAL fallback [M 0x15b830] — the block now drops those states (return false = the
state handler's unhandled). (3) The rescue leg chases a mode-4 dodging captor (walk to his live
position, resupply, reshoot — the east-lane legs' own documented pattern). Both walks green:
walkreal_07000003_live 5/0 with the test UN-IGNORED (top-up 78.6 s, real-damage 25.6 s).

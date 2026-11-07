# WORKLOG

One entry per task, append only (never rewrite or reorder others' entries).

## level-14-door-pair

2026-11-07 08:02 UTC, branch `s/level-14-door-pair` (from origin/master 22d5836b7).

Task: both 07000014 door guards reach the locked door as they do in the game. In the game
the released group-9 pair #778/#780 stop 2.4 apart at (-51.6,41.2)/(-51.2,38.8) [L 2026-10-30];
in the port only #783 routes to the door while the pair wake into no-route combat at their own
spawns and shuffle there (`the_door_guards_hold_combat_no_route_like_the_game`, ignored, in
crates/nf-view/tests/door_guards_locked_door_07000014_live.rs). Plan: read the queue reports
(done/10-level-14-no-route-hold.md, done/10-level-14-door-guard-release.md, the
10-level-14-missing-group-guards report, done/20-drone-nobunching-side-bits.md), confirm the end
state on current master, follow the release channel and the alert/goal that sends the pair, and
how NDrone2_SetDroneAlertRoute 0x154b20 or the no-route arm picks the door side [M]. Fix the
route gap first; NB_SCAN_LIVE only if the pair needs it and walkreal_07000014_live stays green.

2026-11-07 09:50 UTC — the finding: the [L] door pair is the mission-walker pair (#783 + #784 at
Agent, the boot default +0x28 = 2), not the group-9 pair. The group-9 templates #778/#780 spawn at
alertness 1.0 (prop 0x20 = param 19 & 3 = 3 [M bitDescs 0x2c7250 + the static table 0x2f5cc8]), so
the game's own wake fold sends them to ATTACK [M 0x14cbd8:122-155] and their combat route to a
hall-side goal cannot cross the locked door link [M 0x1571b0:262-271] — the port's spawn-parked
0x5f is game-faithful at Operative. The walkers (alert 0.66, prop 0x1d) take the kind-8 MissionPath
to the door node [M 0x15bf18:32-56; D node 124 (-51.4,39.6)]; #784 (param 2 = 0x2) exists only at
Agent and up. Landed: the Agent pin (both walkers hold at the door, no 0x81) + the rewritten ignore
note; no src changes. Runs: 014 door binary 5/0/1ign, walkreal_07000014 9/0/1ign, interactables 10/0,
ai_traces+npc_doors 6/0/1ign, nf-game lib 686/0/5ign. The suite runner was killed twice by the
sandbox (no total line). Live checks: the [L] boot's difficulty, the door guards' ids, the group-9
pair's in-game state chain.

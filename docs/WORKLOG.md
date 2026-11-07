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

# WORKLOG

## level-0a-civilian-slide-cels
Started 2026-11-12 01:24 UTC. Branch s/level-0a-civilian-slide-cels cut from origin/master 08a276ce0.
Task: 0700000a office civilians glide (danim 392 plays with no gait) while in full view because the
sim-side drawn gate (`Drone::drawn`, stand-in for obj+0xFC) fails on the port's bullet-segment ray
stand-in (thin posts, non-pass foliage block). The game sets obj+0xFC from `View_AddObjects`
0x1e62d8:71: the object's world sphere passes `Vision_InView` over the PORTAL-VISIBLE cels, no rays.
Plan: give the drawn gate the game's own room rule (drone's cel in the portal-visible cel set from
the viewer's cel, then the frustum test, no rays), share/move CelGraph support into nf-game, keep
`drone_position_visible` (spawner's reader) separate. Tests: real-input walk to the vantage
(4.59, 0.65, 2.43) cel 29 (red on master: in-view civilian on danim 392 with drawn == false), the
lounge-window patroller through the z -4.71 glass pane, and the guards-0c / 05-castle slide tests
still hold (opaque-occluder case stays undrawn).

## level-0a-civilian-slide-cels-2

Branch `s/level-0a-civilian-slide-cels-2` = origin/s/level-0a-civilian-slide-cels d7e047a08 (origin/master already an ancestor — no further master commits at branch time). Task: the drawn gate (the game's portal-visible room rule, nf-game celgraph.rs, Drones::tick_with_cel_graph) breaks `walkreal_0700000c_eastroute_live::the_ground_route_crosses_to_the_east_side` — after "the ledge to the arc" Bond is at (8.99, 2.01, −222.09) off the 12.4 ledge and the walk wedges at the arc west end (bond 11.36, 1.13, −217.52) with guard #15 in state 0x5f at (11.36, 12.24, −218.92) right at the arc end; with the portal rule that guard now reads drawn where the old ray gate left him undrawn, so his drawn-side behaviour (physics, collision push, movement) runs. Plan: reproduce on this tree, find which drawn-gated arm moves Bond off the ledge (or blocks him), check it against the decompile; if the game does the same, re-tune the walk leg on real input (no writes); then the suite (the drawn bit feeds auto aim, seek targets and BondTalk on every level — list every walk that moved and why).

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

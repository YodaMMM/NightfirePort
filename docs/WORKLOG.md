
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

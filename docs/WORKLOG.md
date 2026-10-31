## level-06-interactables-2 (2026-10-31, s/level-06-interactables-2)

Branch from origin/master (e4550f2e, batch-1031k). Task: every player-touchable kind behaves
like the game — interactables_07000006_live.rs has 3 tests; using interactables_0700000b_live.rs
as the model (one live test per KIND of player-touchable placement — doors, switches/consoles,
fuseboxes, lifts, pickups, breakables, plates, monitors, rigid bodies, SimpleScripts the player
triggers), run the census of 07000006 [D every placement kind], and for every kind a player can
touch, add the live test that drives it through the real path (walk/E/shoot) and asserts the
game's result [M the class's Update/Activate; D its params], with a sister that fails on the
wrong result. Fix any port gap the tests expose, with evidence. Run interactables_07000006_live,
walkreal_07000006_live and nf-game; quote the totals.

## level-06-interactables-2, progress (2026-10-31)

Census re-run [D]: 30 main-map kinds. Player-touchable coverage now: breakables (the 0xE1
barrel chain — test 1; the 0x20 window pane — NEW test 4, the player round breaks it [M
Break_Update 0x12bfd8:13 + 0x1ece30:24-33; D #376 p2 0xf8 → filter 0xFF07 — unlike 07000014's
blast-only p2-0x20 pane]); pickups (the medal gate — test 3; the kind-1 AMMO walk-over — NEW
test 5 on the plain placed box #679 [M 0x18e148 → 0x18e650 case 1 → Player_EquipAmmo 0x1a8d18's
clip-first arm; D #679 params (1, 0x2, 0x23)]; #666 pinned as a script RESERVED slot — its live
activation sits inside the player's touch at tick 2); SimpleScripts (the barrel-roll knock —
test 2). Doors/fuseboxes/plates/ScriptPlayers/hints/Hurt stay with their sibling files (the
header's map); no switch/lift/monitor/music-trigger kind exists on 07000006 [D census]. Runs:
interactables_07000006_live 5/0, walkreal_07000006_live 3/0, nf-game 953/0/9i across 98 binaries.

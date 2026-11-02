# bot-push-01-escort-routes — land the per-bot push gates without breaking 01

Branch from origin/master; merge origin/s/bot-push-per-bot-gates in first.

That branch ports the game's per-bot push gates (Bond is pushed off guards on overlap, +0x4f8 arming polarity), but four 07000001 walks fail with it: following_mayhew_down, the_exit_fade_starts..., the_plate_refuses_bond_alone, walk_from_the_spawn_to_the_exit_with_the_contact. The contact's escort AI reacts to Bond's pushed position and the walker's spots were tuned to the old no-push path.

Do:
1. Re-tune the 01 walks (routes, waiting spots, clear-around shooting spots) for the faithful push — real input only. If a stall is the port pushing where the game does not, fix the port and cite the decompile.
2. All of walkreal_07000001_live, exit_aim_01_live, flank_fall_repro, walkreal_07000003_live green; then every walkreal_*_live and the full suite.
3. In the report, list exactly how Bond's movement changes when he brushes a guard (lift per tick, push distance), so it can be feel-checked in the app.

---
claimed-by: sandbox4 at 2026-11-02T11:57Z

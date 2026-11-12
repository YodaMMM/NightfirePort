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

## level-0a-civilian-slide-cels-2

Branch `s/level-0a-civilian-slide-cels-2` = origin/s/level-0a-civilian-slide-cels d7e047a08 (origin/master already an ancestor). Task: the drawn gate (the game's portal-visible room rule) broke `walkreal_0700000c_eastroute_live::the_ground_route_crosses_to_the_east_side` — the main agent's check found Bond off the 12.4 ledge at (8.99,2.01,−222.09) and the arc-west-end wedge with guard #15 in 0x5f above. FOUND: env-gated bisect on the branch build (NF_DIAG_OLD forces the master ray gate per drone; NF_DIAG_PORTAL_IDX gives the portal rule to one control entry) — the fall reproduces from the ring patrollers plc=405/404 ALONE (each single-handedly fails the test; the other six flipped drones are innocent). First drone divergence t=1699: with the portal rule the patrollers WALK their real danim gait; under the ray gate they were undrawn and `NDrone2_DoAnimation` 0x149f88's not-drawn arm carried them at the fixed 0.02/frame no-gait slide [M] — the changed patrol positions shift the fights, the fights shift the #282 ride landing to (4.9831,−223.2948) — EXACTLY the real-damage tree's landing — and the plain single long "the ledge to the arc" leg overshoots the band's east edge into the measured fall. Both gates are the game's own (the drawn set is View_AddObjects 0x1e62d8:34-71's VisCelList pair [M]; the no-gait arm is 0x149f88 [M]) — the game does the same, so the walk legs are RE-TUNED on real input (no writes): the eastroute test and the 0c whole-mission walk opt into the existing `with_strip_rd_post_ride` mid-ledge stops (the 0c-2 ledger's lever, built for that same shifted landing; comment-only change in walkreal0c.rs naming both callers). Both binaries green: eastroute 1 passed; 0c_live 4 passed + 1 ignored (the pre-existing agent ledger). Sweep (the drawn bit feeds every level): 0a 13+2i, 02 3+2i, 07 6, barrelrow 1, comms 1, upward 2 green; 01 3+4F+1i (the exit/plate walks), 05 16+1F (the d3 sister), 08 4+1F (the d2 sister), 13 3+1F (the rd sister), guard1_rounds 1+1F (the real-health ring fight falls off the ring edge) — the same gait-shift class, ledgered in the report for the next message. The suite did not fit (the disk surgery + the investigation consumed the window; background jobs die between tool calls in this sandbox).

## level-0a-civilian-slide-cels-3

Branch `s/level-0a-civilian-slide-cels-3` = origin/s/level-0a-civilian-slide-cels-2 91cee1b9f merged with origin/master 7dc4c9dcd (clean merge). Task: the -2 sweep ledger's RED walks — walkreal_07000001_live x4 (first the two non-fight walks: the_plate_refuses_bond_alone and the_exit_fade_starts_on_the_exit_tick_runs_the_games_300_frames_then_the_level_loads — a drawn-gate change should not move a plate's filter or the exit fade, so these may be real regressions), then the contact walk sisters; walkreal_07000005 real_damage_00agent; walkreal_07000008 real_damage_agent; walkreal_07000013 real_damage; guard1_rounds_0700000c_live the ring fight (Bond off the ring west edge). For each: name the drawn-gated arm that moved it; a real regression gets fixed (with the address), a fight that only moved gets a real-input re-tune or an #[ignore] naming a task. No draw changes.

## level-0a-civilian-slide-cels-3 (outcome)

The -2 ledger re-measured on the merged tree (-2 91cee1b9f + master 7dc4c9dcd). FIXED GREEN:
guard1_rounds the ring real-health fight — two mid-rim stops (9.0,-204.5)/(6.5,-206.0) r1.4
north of the measured west-edge fall (6.95,1.06,-210.35); guard #1 still dies at the west edge.
The 01 non-fight walks are NOT port regressions — the fade/plate assertions never ran: the hall
doorway freezes (guard #10 in the leaf, #14 on the pickup line) and the plate sister's route
crossed the restart LoadPointActivator (level_exit Some(0x07000001)) after blocker wedges. All
reds are the drawn-gait class [M 0x149f88]: 01 x4, 05 d3 (the ambush pair closes point-blank),
08 agent (world hurt volume), 13 rd (a 50.12 sniper hit), 15 agent, 11 real-damage x3. No
crates/*/src change. METHOD: post-merge binaries must be re-checked by mtime — the first
round ran the previous session's stale binaries and "passed" 01/08/13 wrongly. Left: the
per-test re-tunes (the report's ledger), the 0c family re-run on the merged tree, the suite.

## level-0a-civilian-slide-cels-4

Branch `s/level-0a-civilian-slide-cels-4` = origin/s/level-0a-civilian-slide-cels-3 c82052ad4 merged with origin/master (clean merge). Task: land the portal drawn gate — the walks the gait shift moved, per the -3 report's ledger. Order: (1) walkreal_07000001_live's exit family (one walk, 4 tests: route around the RESTART LoadPointActivator at (-24.06, 6.46, 7.99), the hall doorway stands #10/#14/#15, keep the contact alive); (2) the two that may be more than a re-tune: 08 agent (live-wire hurt volume at (112.98, 13.47, -177.50) the old path avoided — why does the route change, a guard pushing Bond?), 11 base real-damage (AIRBORNE wedge at (21.51, 48.03, 15.89) near door (20.0, 46.8, 16.6) — check whether a walking guard / door contact pushes Bond, name the arm); (3) 05 d3, 13, 15 agent, 11 sisters. Real input only, never lower damage; unwinnable walks get #[ignore] naming a task, no more than three. Suite with tools/test/suite.py in the background, run to completion.

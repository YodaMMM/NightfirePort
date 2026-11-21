# s/g028-cover-walks: the cover-run reds, triaged

**Moves the shared random stream (on top of s/g028-cover's own move): keep on the branch for a stream batch.**
Built on origin/s/g028-cover (fe48425ff).

**Still red (7, all walk scripts whose fights the stream re-dealt; base green, red on s/g028-cover or after the
route-end fix):**

| Test | Failure on this branch |
|---|---|
| walkreal_0700000d `walk_arrival_to_the_c5_door_real_damage` | Bond dies t11495 at (201.4, 16.0, -256.4): roof guards #28/#14/#27 snipe from their cover spots, 51 u |
| walkreal_0700000d `..._real_damage_agent` | dies t4232 at (213.1, 17.4, -234.5): roof guard #33 at (151.4, 22.7, -245.6) lands 3 x 28.0 from 63 u |
| walkreal_07000003 `walk_the_full_route_and_complete_every_objective_real_damage_00agent` | dies t14235 at (14.7, 9.1, -28.9) |
| walkreal_07000006 `walk_the_castle_interior_to_the_party_real_damage_00agent` | dies t21904 at (6.0, 42.0, -32.7) |
| walkreal_07000011 `walk_spawn_to_the_lower_floor_load_point_real_damage_agent` | the walk wedges at the south corridor west (9.26, 1.13, -12.67), 6000-tick cap |
| walkreal_07000013 `walk_the_warm_chain_to_the_front_entrance_and_out_real_damage` | dies t4128 at (3.3, -368.4, 31.7) |
| walkreal_07000015 `walk_spawn_to_the_omega_sector_exit_real_damage_00agent` | dies t5937 at (35.1, 41.4, 24.7), #7/#8 at 12 u |

None of the seven passed any post-intro stand swept (5-2500 ticks, 21-31 values each).

## What a player would notice

A guard running for cover now gets all the way into the spot. Before, any guard whose last route leg to its goal
was 0.4 to 1.0 long stood still beside the last nav node, 0.55 to 0.8 short of the cover spot, for as long as the
fight lasted (measured: 3000 to 7000 ticks on 0700000d and 07000016), half in cover and half out. The same end-of-route
rule applies to every guard route (patrols, mission routes, goal runs): the last stretch now heads straight for the
route's end point instead of creeping toward it.

## The port fault the longer runs exposed [M]

`AINetwork_SetupNextNode` 0x159668, past the last node (index >= count), non-loop arms: clamp the index to the count
and copy the route's dest (route+0x40) into the move target (route+0x60); no creep recalc. The finished creep keeps
`LinkCreep_Increment` 0x159f28 (step >= steps) calling back, so the target stays the dest. The port instead re-crept
target -> dest in 0.4 steps, and `trunc(trunc(len) x 2.5)` [M 0x159cc8] gives ZERO steps for a leg of 0.4..1.0: the
target never left the last node. With the old flat 2.0 arrival this hid (the guard "arrived" first); with the cover
goal's 0.5 a corner cover run parked at the node, the 2.5 %/tick slide pulling it toward the in-cover point 0.8 off
the goal, `remaining` stuck at 0.54-0.77. Trace: an env-gated RunForCover probe (not committed) over the 18 red tests
printed 13 guards parked 300+ ticks on the radius-only branch and none on base.

Fix: `Route::next_node` (nav.rs) sets target = dest past the end and leaves the creep finished.
Test: `nav::tests::past_the_last_node_the_move_target_is_the_destination` (red on the old code: target stayed at
(10, 0, 0), want (10, 0, 0.67); green on the new). `ai_live::cover_run_rounds_the_gate_pillar_like_the_game` is
unchanged (199 ticks, live 198, arrival 0.05 from the live spot). Doc: docs/systems/ai-network-route.md.

The fix alone turned five of the 18 reds green (escort_follow_07000001, fix_npc_doors, 16's chaser punch, 13 warm chain
d2, and 13's plain warm chain when run alone).

## Checked, no port fault found

- 0700000d guard 15: RunForCover <-> Attack loop for 5000+ ticks, `MoveToGoalPosition` code 4 every tick (no goal
  node in the goal's cel). The port's cel-scoped node search matches `AINetwork_NodesForPosition` 0x1586a8 (the
  no-hit retry over cel+0x28 re-walks the same search block; already documented in nav.rs); the game's code 4 also
  goes to Attack (0x169150 default -> case 0x1b). Left alone. Unproven: whether the game's FindCover then picks the
  same node again.
- 0700000d guard 10: hovers 0.51 from its cover goal for 7000 ticks after the fix (strafe 0xd root motion about
  0.008/tick toward the goal against the 2.5 % slide toward the in-cover point). Same code shape as the game
  (0x169150:62-67); the strafe's root speed at that phase is unverified against a live trace. Left alone, unproven.
- 07000001 door #1 (7.9, 5.3, 11.1): Bond's scripted legs wedge on the open shoji leaf at (5.45, 6.67, 12.3) and
  (8.14, 6.44, 11.71) once the hall guards' cover runs leave the door in a different state. Walk-script class (the
  leaf's collision is old behaviour, not touched here); answered by the re-rolls below.
- 0700000d snipers: the roof guards #28/#33 now reach their cover spots at (150.57, 21.7, -248.5/-250.8) and snipe the
  deck walk from 51-63 u (16-28 a hit). Game-faithful cover behaviour; the walk script needs a new answer.

## Walk re-tunes (no game logic changed)

A new walker opt-in, `set_post_intro_wait(n)` (common/damage_log.rs, used by both walkers' `wait_enabled`): Bond
stands n ticks after the intro before the walk starts, a re-roll of the dealt fights. Swept per test over stands
5-2500 (up to 21 values); each pinned value is a measured pass, the passing values are listed in each comment.
| Test | Stand |
|---|---|
| walkreal_07000001 `following_mayhew_down_completes_the_level` | 30 |
| walkreal_07000001 `the_exit_fade_starts_on_the_exit_tick_...` | 90 |
| walkreal_07000001 `the_plate_refuses_bond_alone` | 30 |
| walkreal_07000001 `walk_from_the_spawn_to_the_exit_with_the_contact` | 90 |
| walkreal_07000002 `walk_the_castle_all_objectives_real_health_00agent` | 100 (d3-gated `w.wait`, the walk has no intro wait) |
| walkreal_07000006 `walk_the_castle_interior_to_the_party_real_damage_agent` | 30 |
| walkreal_07000008 `walk_the_whole_mission_on_real_input_real_damage` | 30 |
| walkreal_07000008 `walk_the_whole_mission_on_real_input_real_damage_agent` | 30 |
| walkreal_07000011 `walk_spawn_to_the_lower_floor_load_point_real_damage_00agent` | 90 |
| walkreal_07000012 `walk_spawn_to_the_shaft_exit_real_damage_agent` | 30 |
| walkreal_07000013 `walk_the_warm_chain_to_the_front_entrance_and_out` | 30 (also red on base when run in its binary) |
| walkreal_07000014 `walk_the_tripbomb_route_pick_up_place_and_shoot_it` | 200 |
| walkreal_07000015 `walk_spawn_to_the_omega_sector_exit_00agent` | 30 |
| walkreal_07000015 `walk_spawn_to_the_omega_sector_exit_real_damage_agent` | 30 |
| walkreal_07000016 `walk_the_chasers_defeat_completes_the_mission_on_real_damage` | 30 |
| walkreal_07000016 `..._real_damage_agent` | 150 (the only pass of 16 values) |
| walkreal_07000016 `..._real_damage_00agent` | 300 (the only pass of 16 values) |

Re-pin: `guard1_rounds_0700000c_live::guard1_rounds_at_the_ring_pinned_over_a_real_fight` counted 5 rounds before
the window's first reload; on this stream guard #1 enters the window with a part-spent clip (reload #0 after 4 rounds), so the first reload now counts from
the clip held at the window's first sample (every later reload still counts the full 5; the game's rule is unchanged).

The armour-pickup lever was not tried on the seven still red; the stand sweep was the only lever used.

## Tests run

All with NF_SILENT=1, release, `--features nf-view/movies,nf-formats/movies`, each binary run whole (`--test-threads 2`).

- nf-game: lib (826, the new nav test included), ai_live (38; the gate-pillar run still 199 ticks), and
  cover_selection_live, ai_points_live, aipoint_accuracy_census, guard_reaction_live, grenade_live, stun_grenade_live,
  blast_cooks_off_grenades_live, stuck_guards_sweep, guards_0c_live, door_guard_live, guard_ai_other4_live,
  fix_ai_remainder_live, ai_missing_states_live, drone_portcheck_live, attack_mode_c4_live, zoe_abandon_live,
  bathhouse_live, guard_barks_live, find_opponent_live, dtype_sniper_alert_live, astronaut_guards_live,
  guard_drops_flashbang_live, guard_round_civilian, guard_side_gates_live, nav_kick_live, rope_guard_hitbox_live:
  all green.
- nf-view, 185 binaries: every walkreal_* binary (109), escort_follow_07000001, fix_npc_doors, npc_doors,
  escort_cover, escort_hide_findrun, cover_nodes, cover_nodes_submap, and the 69 other nf-view tests whose names
  carry patrol/route/mission/escort/civil/nav/follow/ally/cover/stuck/guard. Red: the seven walks listed at the top;
  guard1_rounds_0700000c was red and is green after its re-pin (re-run alone). Everything else green.
- Baseline: on the branch's base (the 2.0 cover radius and the old route end, switched in by a temporary env gate,
  not committed) the 12 binaries that went red were all green except walkreal_07000013's plain warm chain.

## Unproven / not done

- The seven walks above still red need hand re-tunes (fights re-dealt; 0d needs a sniper answer). No PCSX2 check
  was possible, so whether the game's guards ever hover just outside a 0.5 cover radius (0d guard 10) is open.
- Only the 07000005 gate run is live-matched for cover arrival; the other levels' cover timing is unverified.

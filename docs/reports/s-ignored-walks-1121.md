# s/ignored-walks-1121: the ignored walk census

No game-code change on this branch (tests only). Nothing moves the shared random stream.

## Census: every `#[ignore]` in `crates/nf-view/tests/walkreal_*.rs` and `walk_*.rs` (base 672884f0f)

| # | binary | test | difficulty | ignore reason (short) | cause | this branch |
|---|---|---|---|---|---|---|
| 1 | walkreal_07000001_live | walk_from_the_spawn_to_the_exit_with_the_contact_real_damage | Operative | nb-scan-all: Bond dies t28813 at (-29.86,6.47,13.64) | walk script: the plate leg's return-fire flank walked into the stairwell pocket | **fixed, un-ignored, green (43.0 hp at the exit)** |
| 2 | walkreal_07000001_live | ..._real_damage_agent | Agent | nb-scan-all: dies t14006 at (-22.47,6.47,13.64) | walk script: ammo and the pocket mouth fight | deeper (t26967), still ignored, reason updated |
| 3 | walkreal_07000001_live | ..._real_damage_00agent | 00 Agent | nb-scan-all: dies t2324 at the hall lane stand | walk script: #7's shotgun at the lane stand | reproduced t2324, untouched |
| 4 | walkreal_07000003_live | probe_mouth_approach_routes | - | probe (no walking) | probe, not a walk | - |
| 5 | walkreal_07000003_live | probe_door8_cluster_load_state | - | probe dump | probe | - |
| 6 | walkreal_07000003_live | probe_saddle_armour_pickup | - | probe | probe | - |
| 7 | walkreal_07000005_live | the_door_band_exit_chains_to_07000006_walked_real_damage_00agent | 00 Agent | nb-scan-all: dies t10830, every gun dry at the walkway pair | walk script: ammo budget | reproduced t10830, untouched |
| 8 | walkreal_07000008_live | walk_the_whole_mission_on_real_input_real_damage_00agent | 00 Agent | nb-scan-all ledger; named a "crouch latch" port bug | named game-logic gap checked: NOT a port gap (see below); walk script | settled, still red, reason updated |
| 9 | walkreal_0700000a_live | the_west_hall_glide_height_matches_the_game | - | settled y 1.0905 vs the game's 1.1285 | game-logic (collision) fidelity, needs a live trace of BL+0x790 | untouched (not a completion blocker: the 0a walks pass) |
| 10 | walkreal_0700000a_live | the_corner_crosses_like_the_game | - | the port wedges where the game slides the corner | game-logic (collision) fidelity, needs the same live trace | untouched (not a completion blocker) |
| 11 | walkreal_0700000c_live | walk_the_whole_mission_from_the_start_to_the_0d_handoff_real_damage_agent | Agent | nb-scan-all: dies t6159 at (53.76,6.01,-158.87) | walk script: Bond reaches the deck dry (clip 1 spare 0); #19 lands 2 x 28.0 at 9.1 u | reproduced t6159, untouched |
| 12 | walkreal_0700000c_live | ..._real_damage_00agent | 00 Agent | nb-scan-all: dies t35413 at (94.20,8.92,-232.05) | walk script | not re-run |
| 13 | walkreal_0700000d_live | walk_arrival_to_the_c5_door_real_damage_00agent | 00 Agent | 23-run ledger: the ride lottery and the yard landing burst | walk script (health budget) | not re-run |
| 14 | walkreal_07000014_live | walk_the_infiltration_to_the_delta_exit_real_damage_00agent | 00 Agent | round-3 ledger: dies in the Kiko corridor | walk script | base run wedges at "the south loop to the west corridor node 5" (6000-tick cap, health 27.5); untouched |
| 15 | walkreal_07000014_live | walk_the_infiltration_to_the_delta_exit_real_damage | Operative | dies t36801, #28/#29 land 65.2 at 4 u | walk script: the node-26 fight emptied the gun at 15.8 u | **fixed, un-ignored, green (48.7 hp at the 07000015 load)** |
| 16 | walkreal_probe0c_northend_live | probe_north_end_route | - | probe | probe | - |
| 17 | walkreal_probe12 | dump_the_lobby_region | - | probe | probe | - |
| 18 | walkreal_probe12b | scan_the_north_wall_gap | - | probe | probe | - |
| 19 | walkreal_probe12c | ss_positions_and_post_e_gap | - | probe | probe | - |
| 20 | walkreal_probe12d | lobby_floor_grid | - | probe | probe | - |
| 21-23 | walkreal_probe12f | dump_the_wedge_region, repro_the_alarm_charge_wedges, repro_bond_in_the_cafeteria | - | probes | probe | - |
| 24-29 | walkreal_probe15 | probe_door32_corner, probe_nav_route_to_exit, probe_door29_lane, probe_south_lane, probe_triggers_and_scripts, probe_armour_pickups_and_west_room_guards | - | assertion-free dumps | probe | - |
| 30 | walkreal_probe15b | probe_ladder_deck_and_north_band | - | dump | probe | - |
| 31 | walkreal_probe15d | probe_disc_tris_near_ladder | - | dump | probe | - |
| 32 | walkreal_probe15e | probe_526_shot_lines | - | dump | probe | - |
| 33 | walkreal_probe15m | probe_door685 | - | dump | probe | - |

`walk_probe.rs` and the other `walk_*.rs` files carry no ignore attribute.

### Grouped

- **Probes (22):** rows 4-6, 16-33. Assertion-free dumps; nothing to complete.
- **Walk-script problems (10):** rows 1-3, 7, 8, 11-15. All ten date from the nb-scan-all guard change (3412a89a3: the spacing scan on every level and CanAltAttack in the fire states). Every one is a real-damage sister; the god-mode walks of the same levels pass. Two fixed here (rows 1, 15).
- **Game-logic fidelity gaps that do not block completion (2):** rows 9-10, the 0a doorway glide height and corner slide. Both ledgers already name the live trace that would settle them.

Levels whose Operative real-damage walk was ignored at base: 01 and 14. Both are green on this branch, so no Operative real-damage walk is ignored any more. The Agent and 00 Agent reds that remain: 01 (Agent, 00 Agent), 05 (00 Agent), 08 (00 Agent), 0c (Agent, 00 Agent), 0d (00 Agent), 14 (00 Agent).

## The named game-logic gaps, checked against the decompile

1. **The 08 "crouch latch" (row 8).** The ledger asked why a crouch press after the hole drop did not stand Bond up. The game: `Player_ChangeSubState` 0x190630 sets the stand/crouch lockout `BL+0x960 = (char)FRAME_RATE` on both the stand-to-crouch and crouch-to-stand changes [M :48-56], and `Player_SSCrouch` 0x1924c0 tests the stand press (`Input_Action(8 or 7, 4)` in toggle mode) only while `+0x960 == 0` [M :84-92]. The port's `Player::lockout` models the same gate. Measured on the port: the fall ends with 45 lockout ticks left, so run 11's presses were swallowed. There is no latch fault. Waiting the lockout out and pressing once does stand Bond, but the stand lifts the root from 13.086 to 13.470 in one tick, back into the terminal's hurt band. The world hits resume and the walk dies at t20944 (crouched: t21070). The standing lower-floor fight the ledger planned is not zap-free. That change was reverted; only the ledger text changed.
2. **The 01 "the sighting flag never clears" claim (the row-3 ledger).** Checked: the only `+0x4F8 & 0xFBFFFFFF` writers are `DroneFunc_HandleImpact` 0x1469b8, gated on `GameState+0xC - 0x7000009 < 2` (levels 09/0A) [M :103-127], and `NDrone2_DSTATE_HostageIdle` 0x15ce60 [M :20]. No other mask clears bit 0x4000000. The ledger's claim holds; this is not a port gap.
3. **Unproven, and left alone:** in the game, the lockout also ends early when the transition anim stops (`AnimScriptIsStopped`, the anim plays at 4x speed [M 0x190198]). Without that, it ends at 59 ticks, not the port's 60 (the top of 0x1924c0 and 0x192310 decrements it and zeroes it when it reaches 1 [M]). The port documents the anim part as a deviation. How long the lockout really lasts in SP needs a live read of `BL+0x960` per frame after a crouch press.

## What changed (tests only)

- `walkreal_07000001_live.rs`:
  - The descent legs ("the stairwell head", "the plate") set `flank_hold_x = -27`. This applies on real damage only, so the topped-up walks are byte-exact. The Operative walk is un-ignored.
  - For Agent on real damage: the 00 Agent walk's west armour box take (armour 50.0), plus `reload_full` before the slab. The walk now dies at t26967 instead of t14006. Pocket guard #17 lands 52.3 in 49 hits from 7.2 u while every eye ray to his parts is blocked.
- `walkreal_07000014_live.rs`: the Operative wake legs set `finish_fights = false` after node 21 (d1 only). Un-ignored. The Agent sister is unchanged and green.
- `walkreal_07000008_live.rs`: the ledger and ignore reason now carry the lockout finding.

## Tests run

- walkreal_07000001_live: `..._real_damage` pass (twice, deterministic), `..._agent` fail t26967 (ignored), `..._00agent` fail t2324 (ignored, base death).
- walkreal_07000014_live: `..._real_damage` pass (twice), `..._real_damage_agent` pass, `..._00agent` fail (ignored, base behaviour).
- walkreal_07000008_live `..._00agent`: run 12 measured as above; reverted.
- walkreal_0700000c_live `..._agent` t6159 and walkreal_07000005_live `..._00agent` t10830: reproduced, untouched.
- Every walkreal_* binary: see below.

## Every walkreal_* binary (NF_SILENT=1, -j 8)

109 binaries, one `cargo test --release -j 8 --test <bin>` each, a 595 s cap per binary.

- 107 binaries green on the first pass.
- walkreal_07000001_live: the run was cut off (rc 127 at 85 s, then the 595 s cap on the re-run, where 5 of its 6 non-ignored tests passed: the two real-damage-free walks, the plate, the hall channel, the exit fade, and the newly un-ignored Operative real-damage walk). The sixth, `following_mayhew_down_completes_the_level`, passes alone in 527 s. All 6 are green; the base suite timed the binary at 720 s.
- walkreal_07000013_live: `walk_the_warm_chain_to_the_front_entrance_and_out_real_damage_agent` failed once (Bond died t4583, drone #5 at 1.0 u). It passed on the immediate re-run (15.6 s). That binary's sources (the test file, common/walker_b2.rs and common/tower2.rs) are identical to base, so this is not a branch regression. It is a nondeterministic red that is worth its own look: the 2026-11-20 suites had it green three times.
- No other reds. Compared with base, the branch adds two greens (01 Operative and 14 Operative real damage) and no new reds.

## Left (the 4-hour stop)

- 01 Agent: the mouth fight against the pocket guard #17. He hits Bond through a line that every eye ray to his parts misses. Either find a stance that sees into the pocket, or fight him first.
- 0c Agent: Bond reaches the deck south with clip 1 spare 0. It needs a scoop before the deck, then the #19 sniper exchange.
- 05, 0d, 14, 08, 0c, 01 at 00 Agent: the ledgers above list their next levers.
- 0a glide and corner: the live BL+0x790 trace named in both ledgers.

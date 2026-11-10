# Multiplayer bots: arrival distance and walk speed

The markers-mp-bots work described two bot rules as ported that are not in the code: the goal arrival distance 2.0 (the port still uses under 1.0) and the walk speed 1.3 unclamped (the port still caps it with `.min(1.0)`). Read both in the decompile (the BOT_* move and goal functions the bots.rs comments cite), port them, and pin each with a test that fails on the old code. Run all mp_* and net_* binaries, then a two-instance host+join check on 127.0.0.1 with bots (no desync).

Branch from origin/master, push to `s/mp-bot-arrival-speed`, report in docs/reports/mp-bot-arrival-speed.md. No draw changes.

---
claimed-by: sandbox5 at 2026-11-10T01:23Z

---
## Report (done, 2026-11-10T02:56Z)


# mp-bot-arrival-speed

Branch `s/mp-bot-arrival-speed` from `origin/master` 0a9d39435. The markers-mp-bots work
described two bot rules as ported that were not in the code: the goal arrival distance 2.0 (the
port arrived inside 1.0) and the walk speed 1.3 unclamped (the port capped the walk with
`.min(1.0)`). Both are now ported from the decompile and pinned with tests that fail on the old
code.

## What the game does (the evidence)

**Arrival 2.0.** `NDrone2_MoveToGoalPosition` 0x1517d8:52-54 returns 3 (arrived) when
`AINetwork_GetRouteDistance(route) < *(f32*)(obj+0x8EC)`. The field is the embedded route's
+0x8C arrival radius (the route lives at obj+0x860), and on the MP bot goal chain only
`NDrone2_DefaultInit` 0x14b300:579 ever writes it: **2.0** (0x40000000). A whole-decompile
writer census finds exactly three other writers, none on a bot: `AINetwork_SetupRouteToPosition`
0x158b10:104 (the SP drones' per-goal radius form — their chain, not `SetupGoalPosition`),
`Player_Init` 0x1910f8:53 (Bond's obj, zeroed) and `Player_Aiming` 0x1a1d28:234 (Bond's scope
speed reusing the same offset — a different meaning on a different object). `SetupGoalPosition`
0x15acf8, `CalcRouteToPosition` 0x15afe0, `ResetRoute` 0x15aa00 and `BOT_init` 0x1249e8 never
touch it. The distance itself is the route's remaining **path** (`GetRouteDistance` 0x156320:
dist(feet, next node) + the link lengths; its exhausted/invalid tail returns plain
`Vec_Dist2D(feet, target)`).

**Walk speed 1.3 unclamped.** `BOT_getMovementSpeedMul` 0x126568 (disasm verified): byte
Drone+0xB6 — 0 → 0x3f333333 (0.7), 1 → 1.0, 2 → 0x3fa66666 (**1.3**), else 1.0. The raw chain:
`BOTSTATE_gotoGoal` 0x1278a8:64,76 passes it straight into `AINetwork_SetupGoalPosition` /
`SetupGoalPositionToObj`, which store it raw at AIPoint+0x14 (obj+0x704; only a 0.0 → 2.0
substitution [M 0x15acf8]), and `DSTATE_BotGotoGoalPosition` 0x171240:55-57 re-feeds obj+0x704
raw into `MoveToGoalPosition`. No clamp anywhere in the chain. The disc's default slot-4 bot
record (`INITIAL_BOT_RECORDS`, identical bytes in the EE snapshots at 0x2DEEAA) has
stats[0x06] = 2 — every default MP bot is a 1.3 bot.

## What changed

`crates/nf-game/src/mp/bots.rs` only.

1. `move_to_goal_position`: the arrival is `remaining < 2.0` (was `< 1.0`), where `remaining` is
   the port route's remaining path when one is live and targets the goal (the creep remainder +
   the links after it + the last-node→dest tail, floored at the straight line), else the
   straight-line 2D distance — the game's own exhausted-route shape. The path form matters: a
   goal item's take radius (`Pickup_Update` 0x18e148:203-204 = bounds + 1.1 → 1.24-1.82 on
   07000023) is under 2.0, so a straight-line 2.0 completion stopped the bot short of every
   pickup and the bots never re-armed (mp_bot_voices_live went red); the path form walks the
   route through the item's take radius exactly like the game's node-to-node walk.
2. `move_to_goal_position`: `steer_to(ctx, dest, mul, true)` — the `.min(1.0)` is gone.
3. `walk_body`: the request's over-stick share survives. The stick share stays at the pad's 1.0
   bound (the movement core's own input law); the >1.0 share scales the achieved step — the
   plane walk directly (`dir·speed·over·BODY_STEP`), the world walk as a post-`tick_ctx` scale
   of the body's horizontal displacement. The vertical is never scaled.

No draw changes. No RNG draws added. The shared Rand stream is untouched (mp_lockstep's
one-extra-draw sister still separates).

## Pins (each verified to FAIL on the old code — old code + new tests run before the fix landed)

- `goal_arrival_is_the_games_two` — 1.5 u out on one storey → code 3 (the old code walked);
  sisters: 2.5 u out still walks, 3.0 u up is held by the port's storey guard [S].
- `goal_walk_speed_mul_is_raw` — the speed-2 request carries 1.3 and the plane step is
  1.3 × BODY_STEP; sisters: speed 1 → 1.0 ×, speed 0 → 0.7 × (both unchanged laws).
- `floor_walk_carries_the_over_stick_share` — on the floor world a speed-2 bot covers > 1.2× a
  speed-1 sister's 90-tick walk (the old code: identical, the cap).

Three existing tests whose constants encoded the capped walk / 1.0 arrival were re-bound with
the citations (`body_walks_the_floor_to_the_pickup` pace 45→30, `goal_roundtrip_pickup_visit`
final-position 1.5→2.5, `attack_fire_shot_at_stationary_player` dropped the 0xEC path-shape
assert — the stub's delivery is an interleaving, not a law: the sighted react chains straight
to 0xCF and the idle-enter attack arm [M 0x171b80:45-49] reaches 0xCF too; the shot, aim, latch
and clip asserts stand).

## Verification runs (all green unless noted)

- `nf-game --lib`: **759 passed, 0 failed**, 5 ignored (includes the three pins).
- nf-game mp_*: mp_lockstep (mp-bots feature) 6, mp_match_rules 18, mp_debriefing 10,
  mp_hud_16 7, mp_hud_pane 6, mp_never_mentioned 6, mp_hud_data 4, mp_status_bits 3,
  mp_teams_flow 4, mp_map_objects_live 4, mp_arenas_live 1.
- nf-view mp_*: mp_world_sync_live 8 (incl. `skyrail_with_bots_hashes_agree`: 2 bots, 3600
  ticks, hashes agree — the bots-online desync gate), mp_scenarios_live 19,
  mp_bot_objectives_live 10, mp_combat_weapons_live 11, mp_defaults_setup 12,
  mp_bots_live 3, mp_bots_moving_live 3, mp_bodies_live 3, mp_bot_voices_live 3,
  local_match_bot_pickups_live 3, mp_death_effects_live 3, mp_16_slots_live 3,
  mp_death_hands_online 2, mp_bot_scan_live 2, mp_bot_weapons_live 2, mp_pickup_take_live 2,
  mp_body_anim_live 2, mp_host_match_live 2, mp_bot_armour_online_live 1, mp_clock_gate_live 1,
  mp_core_live 5, mp_enviro_mods_live 8, mp_debriefing_flow 6, mp_menu_flow 0 red (run, green),
  plus mp_hud_real_live / mp_status_strings_live / mp_respawn_ammo_live / mp_shot_gate_live /
  mp_shot_effects_live / mp_pain_options_live / mp_remote_fx_live / mp_pickups_visible_live /
  mp_fists_skin_live / mp_site_damage_offline_live / mp_site_destruction_blast_live /
  mp_net_lobby / mp_net_slots_post_move_live / mp_online_failures / mp_lobby_one_screen /
  mp_menu_lists_live / mp_screens / mp_hud_16_live — run in the pre-path-arrival battery, all
  green; the path arrival touched only the goal-walk completion distance, and the battery's
  bot/pickup/net members were re-run after it (above).
- nf-view net_* (the two-instance 127.0.0.1 host+join and desync gates):
  net_e2e_loopback_live 2 (host FE → 127.0.0.1:port → join → lobby → match → debrief),
  net_desync_report_live 4, net_match_live 3, net_join_live 2, net_resync_live 3,
  net_match_end_clients_live 4, net_internet_live 1, net_soak_faults_live 2.
- The two-instance host+join check on 127.0.0.1: net_e2e_loopback_live (both instances through
  the real front-end flow) + skyrail_with_bots_hashes_agree (bots online, hash equality every
  15 ticks, pickups taken on both machines) — no desync.

Known red on pristine master, not this branch: `world::tests::flight_into_the_02_shallows_
splashes_the_surface_and_stops_at_the_floor` fails on unmodified origin/master too (verified by
stash); it passed in the final lib run — flaky, pre-existing, untouched here.

## What's left / live checks for the main agent

1. **The game's mul → ground-speed consumption is not located.** The verified consumers of the
   raw mul are the route/AIPoint plumbing (0x15acf8 AIPoint+0x14, re-fed at 0x171240:55-57) and
   the probe offsets (`CanStepLeft` 0x14f640:69-71 scales the lateral probe by the mul) and the
   `DSTATE_AlertToPosition` threshold 0x165270:95 (`mul + 8.0 <= AIPoint+0x10`). The drone's
   per-tick ground step is `Drone_Control` 0x139268:72-90's `pos += sin(yaw)·obj+0x58c` where
   +0x58c = `Drone_AnimInfo[anim+4]` (per-anim, no mul found). The port therefore maps the
   over-stick share onto the achieved step [S, commented at walk_body]. Live check: PCSX2, a
   Quick Game with one speed-3 (1.3) bot and one speed-2 (1.0) bot on 07000023, both walking
   the same long straight link — record both bots' obj+0x30 per second for 10 s; the 1.3 bot
   should cover ~1.3× the 1.0 bot's ground. If the ground speeds are EQUAL, the mul's
   ground-speed reading is wrong and walk_body's over-stick scale should be dropped (the route/
   probe consumption stays).
2. **The pickup-goal walk's take race.** The game completes the goal at path < 2.0
   [M 0x1517d8 + 0x14b300:579] while the take radius is bounds + 1.1 [M 0x18e148:203-204] —
   under 2.0 for most rows (07000023: 1.24-1.82). The port resolves this with the path-form
   arrival (the route's final stretch carries the bot through the take radius). Live check:
   PCSX2, one bot, watch it walk to a weapon pickup on 07000023 — record the bot's obj+0x30 at
   the tick the pickup's state flips to taken (Pickup_Handler) and whether the goal's route
   code (BOT_vars goal+0x48) reads 3 that tick; if the game takes the item while the route code
   is still 1 (still walking), the path-form reading is confirmed; if it completes the goal 2.0
   short and NEVER takes small items, the port needs the completion to leave the walk running.
3. The `[S]` storey guard (`dy.abs() < 2.0`) on the arrival: the game's route distance is
   planar, but its route is network-bound; the port's straight-line fallback could complete
   across storeys without the guard. Live check: a goal on a balcony directly above the bot —
   does the game's bot walk to the stairs or declare arrival?

## Usage

Context window / token figures for this session are not visible to me (no token or cost meter
is exposed in this environment). Wall clock: branch cut 01:27 UTC, final push ~03:00 UTC
2026-11-10.

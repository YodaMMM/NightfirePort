# pretrans-arms-3 — the PreTransCallback root zero, rebased onto solid guard bodies

Branch s/pretrans-arms-2 (616d4b0e1, docs/reports/pretrans-arms-2.md) ports the per-frame +0x38
root-motion zero and its setters. This task rebased it onto current origin/master
(0a9d39435 "Guards and corpses are solid to Bond" + e493b19cd "Blast debris spreads across the
line of sight") and fixed the remaining reds as a player would. No sim/src change was needed:
the sim code is exactly 616d4b0e1's, on top of master. The branch is s/pretrans-arms-3
(tip: the rebase cherry-pick 3c6a8e503 + WORKLOG/report commits).

## The rebase

- s/pretrans-arms-3 = origin/master 0a9d39435 + cherry-picked 616d4b0e1 (3c6a8e503).
- One conflict, in the shared return-fire stand (crates/nf-view/tests/common/damage_log.rs):
  master's `fire_shield` cone (opt-in) and pretrans-arms-2's `return_fire_shield_blocks` fold
  (opt-in) landed on the same spot. Both kept — the `fire` gate reads `!blocked && !shield_line`,
  so each walk's own opt-in flag drives its own shield. walkreal_07000001 (the `fire_shield`
  user) and the escort walks exercise both paths.

## The named reds: both fixed by the rebase itself

- `walkreal_07000002_live walk_the_castle_all_objectives_real_health_00agent`: **green**. The full
  castle route walks to the exit in 25570 ticks; at the geisha-2 plate the returned-fire round on
  the captain #17 costs **0 health** (armour 32.0 → 20.0 absorbs it — on the -2 tree this was the
  lethal 9.8 on Bond's last 9.8). Master's solid bodies changed the geisha-2 doorway engagement.
- `walkreal_07000003_live walk_the_full_route_and_complete_every_objective_real_damage` (+ the
  `_agent` variant): **green** (378 s run, 2 passed). The door-8 mouth leg no longer bleeds the
  ~40 health the -2 tree lost; the exit-lane shotgun #16 never gets his kill.

No damage, health, pin or ignore was touched for either.

## The walkreal sweep on the rebased tree (every binary, one per call)

Green (21 binaries, pass counts): 02 (1), 03 (2), 04 (9), 06 (6), 07 (5), 08 (5), 0a (13), 0b (7),
0c_barrelrow (1), 0d (1), 11_route (1), 12 (4), 13 (3), 14 (7), 15 (7), 16 (15), 1b (16), 41 (15),
41_wall (2), 4a (4), 4a_satchel (2). nf-game lib suite on the merged tree: 799 passed, 0 failed.

Red (6 binaries, 7 tests) — **all six are green on both parents** (the -2 report's walk list, and
bare master 0a9d39435 spot-checked for 01's real-damage pair: 2 passed there). They are the
interaction class: the root zero changes guard timing/positions, and master's solid bodies turn
those shifted positions into hard blockers for the walks:

1. `walkreal_07000001_live` (3 tests):
   - `following_mayhew_down_completes_the_level`: the AND-touch plate never fires — "contact
     touched_by=false bond touched_by=false": the parked contact and Bond both miss the plate
     under the new dynamics.
   - the two real-damage escort walks: Bond is drained to 2.8 hp by the hall phase and dies at
     the stairwell head (tick 10682, 46 hits for 100.0). Precise mechanism [L both traces]: the
     hall flank's deeper nav leg wedges one step early on the rebased tree ("flank 1 nav 2" vs
     the -2 tree's "flank 1 nav 3" — a solid hunter stands on the leg), so the 24-hp shotgun
     captain #7 is fought LAST from the wedged node, and his planted 8-pellet volley lands
     48.0 (8 × 6.0) in one window on the drained Bond. On -2 the captain engaged FIRST at
     full hp (cost 7.5); on bare master he skates and misses. Note: #7's weapon datum reads 0x0
     at the fight (the volley rides the script side), so the existing weapon-based
     captain-first order (agent_order) cannot see him.
   - Fix attempts made and REVERTED (unproven): applying agent_order at d1 and adding a
     24-hp class to its key — the loop order did not change (the captain is a dynamic retarget
     inside the phase, not a loop member), and a shared stand-level change does not fit this
     window. The lever for the next round: an opt-in volley-carrier-first retarget in the
     return-fire stand (lesson-7 flag, set by the 01 walk only), or a wedge fallback that
     fights from the door angle when the flank's nav leg is solid-blocked.
2. `walkreal_07000005_live the_door_band_exit_chains_to_07000006_walked_real_damage_agent`:
   the wall-hug crawl prompt wedges at the 6000-tick cap (bond (25.71,6.61,74.65), target
   (25.50,30.96,71.00), segment t 0.37; health 35.6, no drone or corpse within 3 u — the earlier
   fights' shifted timings leave him approaching from a worse angle).
3. `walkreal_07000009_live the_alarm_switch_fails_objective_1`: the walk never reaches the
   ch-124 switch's press reach by pure walker movement.
4. `walkreal_0700000c_live walk_the_whole_mission_from_the_start_to_the_0d_handoff` and
   `walkreal_0700000c_eastroute_live the_ground_route_crosses_to_the_east_side`: walking east at
   wire #282's B end never grabs it (sphere r 5.05 [D probe blk]) — the same reach miss in both.
5. `walkreal_07000011_live walk_spawn_to_the_lower_floor_load_point_real_damage`: wedged walking
   through the south-band doorway (bond (21.51,48.04,15.89), target d 10.97, segment t 0.05,
   grounded=false, door open, no drones within 3 u).

None of these reds is a damage/health/pin/ignore issue; all are walk-vs-changed-world wedges or
the one combat drain above. No draw code was touched.

## What's left

- The 6 interaction reds above, one walk-repair round each (the -2 round's pattern: per-level
  walk repairs with the trace, no sim changes). 01's lever is spelled out above; 05/09/0c/11
  need their wedge dumps replayed against the shifted approach paths.
- Optionally re-run the -2 tree's repair levels (01/14) after that round.

## Live checks for the main agent

1. [S] In the real game at 07000001 (Operative), open the hall door from the meet room and let
   the captain (#7, 24 hp) fire a full volley at a stationary Bond ~3 u away: the game's planted
   captain should land his volley about as the port now does (the root zero is game-accurate
   [M 0x13b490:85-118]) — the walk repair must not be tuned against game behaviour.
2. [S] gdbwatch on Drone+0x38 for a hall guard during his aim stand: expect 0 written every
   frame at PreDroneControl (0x148db0:23) and the PreTransCallback zero pending only when the
   setter re-raises it — confirms the interaction reds are walk-side, not sim-side.

## Usage

not visible to me — this environment shows no token, credit or context-window figures for the
session (no usage panel is exposed to the agent).

## ntfy failed

All three fallbacks failed — ntfy.sh is unreachable from this sandbox (github.com works; the
report push in the same minute succeeded):
1. `curl -sS -m 20 ...` → `curl: (28) Connection timed out after 20002 milliseconds`
2. `python3 urllib` → `urllib.error.URLError: <urlopen error [Errno 101] Network is unreachable>`
   (socket connect to ntfy.sh:443)
3. `wget -q -O- --post-data ...` → no output, killed after the 30 s tool timeout

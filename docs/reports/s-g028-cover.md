# s/g028-cover: guard cover (G-028)

**Moves the shared random stream, and 17 scripted-walk tests go red.** A guard's run for cover now arrives under
the cover goal's own radius (0.5) instead of 2.0, so cover runs are longer and every later draw shifts. Keep on the
branch for a stream batch; the reds below need triage before any merge. `guard_takes_cover_like_the_game` re-pinned
to seed 2 (16 now holds the 90-tick lean).

## What a player would notice

A guard running for a corner cover node now goes all the way round the obstacle and into the spot, instead of
stopping about 1.5 units short and sliding the rest. On 07000005's gate guard the run now takes the live path: strafe
east of the pillar, back off west along it, strafe north round its corner (199 ticks, live 198; arrival 0.06 from the
live spot). Before: 148 ticks, no north leg, arrival 1.5 south of the pillar.

## Cause [M]

RunForCover passes `Drone+0x704` (the goal radius `FindCover`'s `SetupGoalPosition` stored, 0.5) to
`MoveToGoalPosition` (disasm 0x169240/0x169250 `lwc1 $f12, 0x704`); `SetupRouteToPosition` stores it at route+0x8C
(0x158b10:104); `MoveToGoalPosition` (0x1517d8:49-52) and `AINetwork_FollowRoute` past the last node (0x15b110:20-22)
both arrive under route+0x8C. The port's `Drone::move_to_goal` built every goal route with a flat 2.0, so the
follow arrived at 2.0 regardless of the goal. Fix: RunForCover builds its route with `goal_arrival`. The RunForCover
slide (route < 1.0, 2.5 %/tick toward the in-cover point) is reachable now, as in the game. The game passes the goal
radius from every `MoveToGoalPosition` caller (also `ReFindMissionPath` 0x153a30, disasm 0x153ce4/0x153e64); the
port's other callers still use 2.0. Widening it was tried: same reds plus two more on 15, so it was left out.

## Checked, no change needed

- Per-target slots: only the player ever holds a target slot in single player (`AllocateTargetID` 0x141f28 is team-1
  only; a team-1 opponent is always the player, `FindOpponent` 0x1424e0:101-118). Bond-only is exact. Three
  deviations, none visible in play, are listed in docs/systems/drone-cover.md "Target slots".
- Per-level cover node counts (main map, `Level::parse` over `LEVELS`): 01 12+8, 02 3+6, 03 16+8, 05 18+0, 06 80+34,
  07 22+5, 08 17+25, 0b 0+11, 0c 8+78, 0d 56+68, 11 2+24, 12 7+29, 13 19+18, 14 247+0, 15 6+11, 16 38+26
  (corner+low); 04, 09, 0a, 1b and the 2x/4x levels have none. Switch-gated nodes: 06 two, 15 one.
- `Drone_CoverLowNode` 0x137340 sets low flag 2 (p3 = 1) only when p0 = 2; the port sets it for any p0. No disc node
  has p3 != 0, so this is unreachable and was left alone.

## Not done

- `+0x538` blend timing for non-call anims: motion/blend work, out of scope.

## Reds (new; green on the old code unless noted)

Narrowed change, `cargo test --no-fail-fast`: escort_follow_07000001 (1), fix_npc_doors (1), walkreal 01 (2, the
contact dies at tick 31880; this binary was not run on the old code here, so its baseline is unproven), 02 (1, the
geisha group trigger), 03 (1), 06 (1), 08 (1), 0d (2), 11 (1), 13 (1 new; 2 more already red on the old code), 15 (2),
16 (3). Most are real-damage deaths or walk tick caps. On 01 Bond's scripted walk wedges at (3.4, 10.4) on the way to
the corridor plate after the earlier fights go differently. Unproven: whether any of these is a port fault the longer
cover runs expose, or only scripts tuned to the old short runs. Green: walkreal 05, 07, 0b, 0c, 12, 14, cover_nodes,
cover_nodes_submap, escort_cover, escort_hide_findrun, npc_doors.

## Tests

- New: `ai_live::cover_run_rounds_the_gate_pillar_like_the_game` (red on the old code: 150 ticks; green: 199).
- nf-game (with the wide variant): lib (825), ai_live (38), cover_selection_live, ai_points_live, aipoint_accuracy_census,
  guard_reaction_live, grenade_live, stun_grenade_live, blast_cooks_off_grenades_live, stuck_guards_sweep,
  guards_0c_live, door_guard_live, guard_ai_other4_live, fix_ai_remainder_live, ai_missing_states_live,
  drone_portcheck_live, attack_mode_c4_live, zoe_abandon_live, bathhouse_live, guard_barks_live,
  find_opponent_live, dtype_sniper_alert_live: all green. Narrowed variant: ai_live (38) and
  cover_selection_live green; the rest of the nf-game list was not re-run.

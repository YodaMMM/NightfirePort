# s/kick-actionanim-timer

Moves the shared random stream (guard state timing changes on every level with door nodes).
Two commits: the first is green on every binary run (01 7/0/1, fix_mission_live 4/0, 1b 17/0/2 re-run on it alone); the second (lead-hide + astronaut) reds
walks on 01 and 1b and needs a re-pin before merge.

## What changed (crates/nf-game/src/drone.rs)

Timer-C arms that the game writes as `GameState+0x34 + k·FRAME_RATE_INT` (60 in play) or
`k·FRAME_RATE_DIV` (1.0 in play), which the port armed as a raw `frame + k` or a halved rate:

| Commit | State | Arm | Old | New | Evidence |
|---|---|---|---|---|---|
| 1 | KickObject 0x82 (0x1684c0) | ENTER | +2 | none | [M] case 1 falls into case 0, no write; RouteMsgDCV clears +0xCC0 on the change [M 0x171df8:185] |
| 1 | KickObject 0x82 | timer C | +2 | +120 | [M] case 0xc, disasm 0x168540-64 (lw 0x30d0cc FRAME_RATE_INT, sll 1) |
| 1 | ActionAnim 0x83 (0x168628) | timer C | +2 | +120 | [M] case 0xc, disasm 0x1686c0-e0 |
| 1 | OpenDoor 0x81 (0x1681f8) | ENTER, timer C | +2 | +120 | [M] LAB_001683ac, disasm 0x1683a4-c8 |
| 1 | StandFiddle 0x84 (0x1687c0) | ENTER | +5 | +300 | [M] case 1, disasm 0x168830-58 (x5) |
| 2 | AllyLeadHide 0x1f (0x15ec08) | UPDATE exit | +2 | +120 | [M] case 3, disasm 0x15eccc-ec |
| 2 | AstronautCombatMove 0xbf (0x174550) | ENTER | +150 | +300 | [M] :20, disasm 0x1745a4-ec |
| 2 | AstronautHit 0xbe (0x174690) | ENTER | +15 | +30 | [M] :16, disasm 0x174710-30 (FRAME_RATE_DIV x 30.0f) |

Checked and left alone (already right): SmokedOut_Loop raw +5 [M 0x16b878], Taser re-arm
FRAME_RATE_DIV x 5 [M 0x162b98:89], Surrender_Anim FRAME_RATE_DIV x 15 [M 0x1626a0:24,76],
SniperReload FRAME_RATE_DIV x 20 [M 0x15ff80:19].

## Where the states are reachable

Route-node census (`Level::parse` over `nf_view::dev::LEVELS`, node flags raw[0]):
door nodes on 01 02 03 05 06 07 08 09 0a 0b 11 12 13 14 15 16; kick pairs on 0c and 12;
no action (8) or fiddle (0x40) node on any level, so ActionAnim and StandFiddle are not
reached through the route dispatch on shipped levels. Astronauts: 1b. Lead-hide: the 01 escort.

## Tests

`drone::timer_units_tests` (nf-game lib, 6 tests): all 6 fail on the old drone.rs and pass on
the new.

Run on the full branch: nf-game nav_kick_live, astronaut_guards_live, vision_bit_census,
enemy_accuracy green; nf-view enemy_accuracy_live, npc_doors_live, fix_npc_doors_live,
door_guards_locked_door_07000014_live, escort_hide_findrun_live, escort_cover_live,
escort_follow_07000001_live, fix_guard_misc_live, fix_actors_live, walkreal 02 03 05 06 07 08
09 0a 0b 0c 11 12 13 14 15 16 green.

Red on the full branch, bisected:
- AllyLeadHide 120: walkreal_07000001_live 4 failed (the contact dies: restart exit;
  following_mayhew_down; the exit fade) and fix_mission_live the_07000001_exit_works_end_to_end.
  With only this arm back at +2, 01 is 7/0/1 and fix_mission_live 4/0.
- Astronaut timers: walkreal_0700001b_live walk_arrival_to_drakes_defeat_on_real_health (Bond
  dies in the console run). With only those back, 1b is 17/0/2.

## Unproven

Whether the 01 contact's death after the longer hide is the game's own outcome on that walk
or a second port difference exposed by the faithful 2 s hide; only a live 01 escort run can say.

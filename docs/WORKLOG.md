
## escort-doors-reenable

Branch: s/escort-doors-reenable (from origin/master 7e35bd77)
Goal: un-ignore fix_npc_doors_live::the_escort_opens_the_interior_doors_on_his_route on master;
the harness branch showed it green on the older tree, but the FindOpponent alive filter/LOS budget
and the stuck-guard side-step probe merged since move it (fails ~line 249). Find which change moves
the escort/walker, fix port-or-test [M], keep it un-ignored.

Progress:
- Branch created, test located (crates/nf-view/tests/fix_npc_doors_live.rs:91 ignore, asserts at 245/252-255/258-262)
- Setup: game files re-downloading (sandbox reset lost work/ + ISO)

## escort-doors-reenable (progress)

- Test un-ignored on master 7e35bd77: green 4x tick-identical (door #1 +9542 bond 4.7, door #2 +54215 bond 16.3, escort settles x -20.05, 44.5 s) — the reported line-249 failure (assert bond_d1 > 2.5) does not reproduce
- The named merges (b4cdd507 find-opponent-exact, 1c5e71f8 probe_move) are in master and moved the route (~2x the ticks vs the harness branch's 26543436 run) but all three can-fail asserts hold
- Neighbourhood binaries green on the final tree (26 passed / 0 failed / 1 ignored): npc_doors_live, escort_cover/follow/hide_findrun, walkreal_07000001_live, find_opponent_live, stuck_guards_sweep, doors_live, doors_01_live
- Full suite blocked by sandbox disk (a wrong target-dir arg nested duplicate build trees; recovery hit the 9.9 GB ceiling twice) — per-binary runner-protocol runs stand in; details in the report

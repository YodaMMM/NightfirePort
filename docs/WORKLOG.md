
## escort-doors-reenable

Branch: s/escort-doors-reenable (from origin/master 7e35bd77)
Goal: un-ignore fix_npc_doors_live::the_escort_opens_the_interior_doors_on_his_route on master;
the harness branch showed it green on the older tree, but the FindOpponent alive filter/LOS budget
and the stuck-guard side-step probe merged since move it (fails ~line 249). Find which change moves
the escort/walker, fix port-or-test [M], keep it un-ignored.

Progress:
- Branch created, test located (crates/nf-view/tests/fix_npc_doors_live.rs:91 ignore, asserts at 245/252-255/258-262)
- Setup: game files re-downloading (sandbox reset lost work/ + ISO)

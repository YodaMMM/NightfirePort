# s/walks-1121a: the three walks the stream batch left ignored

Base: `origin/s/stream-batch-1121` (52f1b6393; not yet on master when this branch started).
Does not move the shared random stream: every change is in the three walk test files, and
no game logic was changed.

All three walks are un-ignored and complete:

| Test | Was | Now |
|---|---|---|
| `walkreal_07000002_live::walk_the_castle_all_objectives_real_health_00agent` | stalled before the bunker (ch 0x81 never raised; the bunker-door legs looped for 300k ticks) | completes, 47110 ticks |
| `walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_real_damage_00agent` | dead at t10165 at the lane mouth | completes, the exit Load Point fires, end health 42.6 (57.4 across 18 hits) |
| `walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage_agent` | dead at t20311 at the yard landing | completes in 33203 ticks, the hand-off to 0x0730000d, end health 10.7 |

## What changed (test side only)

### 02, 00 Agent
- `loot` (at-the-feet drops only, <= 5 u, same floor) walks the drop straight at
  difficulty 3 instead of a nav leg. The nav leg to the lane drop (-56.2, 4.4, 0.7), 3.1 u
  away, routed back east over the west wing, left Bond on the ledge north of the lane, and the
  bunker-door legs looped on 3000-tick caps. Only the one d3 test in the file runs this arm.

### 03, 00 Agent (every arm is `difficulty >= 3`)
1. The spawn lawn body armour first: the placed kind-3 pickup (50) at (-10.92, 5.34, -38.05),
   15 u south of the spawn [D the placed pickups]. Before, Bond reached the lane mouth with no
   armour and the mouth stands left 38.2. With it, the mouth stands end at 100.0 health.
2. The window sniper #6 (0x1e at (26.0, 9.8, 0.4)) gets a sniper-first aimed stand from the
   #7 line spot, right after the #7 stand. He had landed 34.6 + 48.0 + 17.4 on that spot and
   the saddle walk.
3. The trio-wake sweep's pick count 4 -> 100. `clear_exposed_nearest_03` spends a pick on each
   10-tick wait, so 4 picks held about 20 ticks and the loot walk then met #4 and the captain
   #3 (100 -> 2.4). The sweep's own 900-tick wait budget now bounds it. (40 picks also cleared
   the wake but died later at the through-gate stand.)
4. The lane south leg starts on the lane node (10.65, -21.62): the lane-north loot can end at
   (19.8, -21.9), and the nav from there ran down x ~14 to the one-way door (14.0, -30.3).
5. `scoop_drops_03` at 00 Agent: the same-floor band 1.5 -> 1.2, and a drop is scooped only
   when the straight line to it passes `move_test`. #8's drop on the building ledge
   (14.41, 8.09, -28.13) and #11's drops east of the lane wall were reachable only round
   through the lane doors, and the way back was the one-way door: five moat legs wedged on it.

### 0d, Agent
- After the annex drop, the walk takes the annex floor's own body armour: the placed kind-3
  pickup at (192.82, 10.79, -272.03), 8.5 u north of the landing [D the placed pickups;
  pickup 2 in the level's list]. The arrival is 1.3 (the touch is the radius + 1.1 sphere
  [M Pickup_Update 0x18e148:203-214]). The way out runs south-west to the control corner,
  because the direct lines back to the landing and to the bay enter wedge on the wall at
  z ~-275.5. Armour 0 -> 50 at 54.8 health. The yard sweep then kills five and the walk completes.

## Levers measured and dropped (not in the tree)
- 03: an aimed hold at the lane mouth for #10. He holds unexposed at (20.5, -20.6) in state 0x96
  for 2400+ ticks, so the hold never fires. With the armour, a 600-tick hold at the lane node
  died earlier.
- 03: skipping `use_door` when the lane doors are already open. This fixed a wedge before the
  armour, but with the armour the doors are shut at that point, so it was dropped.
- 03: a random sweep of 24 runs over 1-3 `NF_WALK_PAUSE` stops and the lever toggles. None
  completed. They died at the trio wake (t53-57k) or on the scoop leg.
- 0d: the 00 Agent container drop at Agent. The hop flew long (fall timer 100) and the landing
  cost 33.3. It died at the exit landing.
- 0d: the strip-side yard armour (205.41, 0.11, -205.31) at Agent. It was taken, then Bond
  died at the ladder top at t6772, the same pocket-visit wake the 00 Agent ledger records.

## Tests run (NF_SILENT=1, release, -j 8)
- Each of the three walks alone, green on the final tree. The ignored versions fail on the base
  (the repro runs: 02 the 0x81 assert at line 1659 after 370 s; 03 death t10165; 0d death
  t20311).
- Every walkreal_* binary (109 binaries, `--test 'walkreal_*' --no-fail-fast`,
  `--test-threads 4`): 283 passed, 0 failed, 33 ignored. No reds, so there is nothing to
  compare against the base. The ignored count is the stream batch's 15 already-ignored walks
  plus the probe binaries' ignored probes. The three walks above are no longer among them.

## Unproven
- Each walk is deterministic on this tree, but the next stream shift can re-roll it.
- 03 still burns about 29 3000-tick caps on the servant 0 and servant 1 navs late in the walk.
  The walk completes anyway. Whether master's green run had the same caps was not checked.

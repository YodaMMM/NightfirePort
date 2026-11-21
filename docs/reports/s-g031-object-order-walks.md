# s/g031-object-order-walks: the walk re-tunes for the same-tick lift carry (G-031)

**Moves the shared random stream on routes with a carrier** (kept on this branch with s/g031-object-order).

## Remaining reds

None in the walkreal_* level binaries (28 + celprobe) on this branch. 0700000d's
`walk_arrival_to_the_c5_door_real_damage_agent` stays ignored as on master (run here with
`--ignored`: still dies at t20298 at the yard landing after the gantry ride, the master ignore reason).

## Base

Rebased onto c3b9de140 (the stream batch). The only conflict was the GAPS G-031 row: master's
G-029/G-030 rows were kept, and the G-031 row is master's text with this branch's object-order paragraph.

## What changed

- **0700000b Operative recording re-pinned** (`tests/replays/0700000b_operative.bin`, 17779 -> 17781
  ticks). Why: the carry now uses the cab's move from the same tick. That shifts Bond's paternoster ride by
  under 0.0001 u (the duct arrival is 6.57 -> 6.56), and two more ticks are spent on the route.
  `replay_parity_0700000b` passes on the new file.
- **0700000b Agent / 00 Agent: a stand at E-plate #166 before the use** (120 / 300 ticks; env
  NF_0B_PLATE_WAIT2/3 overrides it). The same sub-ulp ride shift re-rolled the fights. On the exec floor the
  plate fight killed only #0, where master killed #0 and #1. Both walks then died on the roof: d2 at t17712
  at the jump edge, d3 at t17433 at the deck key, from #4's 0x18 rounds. The stand re-times the roof
  cut-scene and the deck fight. Sweep [L]: d2 completes at 60 and 120 (end health 18.8 / 24.4) and dies at
  30, 200 and 300; d3 completes at 30, 60, 200 and 300 (end health 2.9 / 26.5 / 47.5 / 58.0) and dies at 120.
  The kind-3 armour on the exec floor (17.53, 47.68, 28.46) is takeable, armour 0 -> 50 [L]. The detour
  stretched the walk by about 7000 ticks: d3 completed and d2 died, so the armour lever was not shipped.
- 0700000d: with the stream batch already on master, the carry left both of its runnable walks green. No
  change was needed.

## s/g031-trigger-lag on top: not joinable now

Its drone.rs change (plates read last tick's player touchers) was applied on top of this branch and every
walkreal_* level binary was re-run: **27 reds in 13 binaries**:
01 (1), 02 (1), 03 (1), 05 (2), 06 (2), 09 (1), 0a (4), 0b (3), 0d (1), 11 (2), 13 (1), 15 (3), 16 (5).
Some of these are not dice re-rolls: 09 `the_alarm_switch_fails_objective_1`, and 0a's warm (no damage)
`walk_office_floor_qworm_keyboard_hatch`. Those need a look at the plate timing itself before any re-tune.
The 27 were not re-tuned.

## Tests run (NF_SILENT=1, cargo test --release -j 8)

- Every walkreal_* level binary + celprobe, carry only: all green except 0b (2 red before the re-tune).
- After the re-tune: walkreal_0700000b_live 8/8, replay_parity_0700000b 1/1, g031_object_order_live 1/1.

## Unproven

- The stand lengths are dice, not game facts. Neighbouring values die (see the sweep above).

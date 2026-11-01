# 07000011 start-corridor guards in PCSX2 (2026-11-01)

The question: on a fresh 07000011 load with Bond idle at the start (spawn #146, (−0.40, 47.85, 25.83)),
do the office guards #126/#127 walk into the corridor and attack him, and at which difficulty?

## Method

PCSX2-gdb, fresh boot of the ISO (no save state), then `tools/trace/goto_level.py 07000011 --settle 1`
(the game's own `ResetMap_LevelToLoad` path). A PINE loop sampled every drone within 30 units of Bond
every 50 in-game frames (GameState+0x34, relative to the frame after the load): `obj+0x30..0x38`, the
state `Drone+0x10C` (mirrored to `obj+0xF4`), `+0x5A2`/`+0x5A4`. Runs with Bond's health held
(BLData+0x894 = 100, +0x8B0 = 50, damage events logged before each write) and runs without (the death
frame). No pad input at any point. For d1/d3/d0 the difficulty global **0x2A3790** (GameState+0x28 hex,
Ghidra's decimal `_40_`) was written before the load; `Drone_Create`'s gate reads it at load
[M 0x136be0:49].

## Results [L]

- **The fresh-boot difficulty is 2** (0x2A3790 = 2 read live; the boot default [M bootup 0x1c7e78:25]).
  A capture made with goto_level after a fresh boot ran at **Agent (d2)**.
- The menu stores 1/2/3 (Operative/Agent/00 Agent) [M C_SBNFDFCTY 0x216ca8:15]; d0 is not selectable.
  With 0x2A3790 = 0 forced the level had no drones in the list at all.
- **The guards approach at every selectable difficulty.** Both start in GoToGoal 0x63 on frame 1 and
  reach the corridor south of the spawn by frame ~150:

| diff | guard (spawn) | f1 | f200 | f700 | f1500 |
|---|---|---|---|---|---|
| 1 | #126 (9.73, 11.92) | 0x63 | (3.01, 17.67) 0x74 | (0.61, 17.06) 0x68 | (−1.95, 17.70) 0x69 |
| 1 | #127 | absent (p2 = 2 > 1) | | | |
| 2 | #126 (10.13, 11.70) | 0x63 | (3.04, 17.69) 0x74 | (2.77, 17.64) 0x73 | (2.36, 17.26) 0x73 |
| 2 | #127 (−11.11, 12.43) | 0x63 | (−1.40, 19.91) 0x63 | (−1.25, 20.07) 0x68 | (−1.25, 20.08) 0x69 |
| 3 | #126 (10.09, 11.72) | 0x63 | (3.01, 17.77) 0x74 | (−0.71, 17.09) 0x74 | (−2.41, 17.39) 0x68 |
| 3 | #127 (−11.07, 12.46) | 0x63 | — | (−1.23, 20.05) 0x73 | (0.41, 20.02) 0x68 |

  (#126 is identified by the d1 roster: the only one of the two with p2 = 1.) They hold 5.8-9 units
  from Bond, cycling StepAimLeft/Right 0x73/0x74, AimStandFire 0x68 and 0x69, and fire at him.
- **Damage per hit:** 10 (d1), 12 (d2), 20 (d3). **Idle Bond dies** at frame 1113 (d2) and 878 (d3);
  at d1 one run took a single hit in 2000 frames (90 hp), the held-health run logged 10 hits in 1600.
- A civilian (start 0x10 → CivilianMission 0x16 → 0x64 → CivilianHiding 0x13) also walks from
  (3.03, 15.91) to (3.7, 21.1) at d1-d3.

## Conclusion

The game's own guards walk into the corridor and kill an idle Bond at Agent and 00 Agent; an empty
corridor over f200-f3172 cannot be a fresh idle start at a selectable difficulty. The port matches in
the parts that matter (both guards from tick 1 in 0x63, the same corridor spots, the same per-hit damage,
death at tick 1102 (d2) / 608 (d3)). Pinned by `crates/nf-view/tests/start_corridor_guard_07000011_live.rs`.

Still different: the port's guards hold AimStandFire 0x68 once in position, while the game's alternate
the StepAim 0x73/0x74 sidesteps with it (the cause is not traced here); at d1 the port's #126 first stops in AimStand 0x67 at
(3.06, 15.74), ~10.7 from Bond, at t200 (the game's is at (3.01, 17.67), 8.8 away) and closes in by t400.

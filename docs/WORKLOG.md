
## real-health-agent-difficulty-12-13-14-15 (started 2026-11-03 15:00 UTC)

Branch from origin/master (f9c1d69c — the 01-07 branch is already merged, nothing extra to merge).
Task: the Agent (difficulty 2) sisters of the real-health walks of 07000012/13/14/15, the 01-07
idiom (load_with_carry_difficulty the menu way [M C_SBNFDFCTY 0x216ca8:15], the Drones::difficulty
assert at load, the difficulty-1 call sites byte-identical), the 01-07 report's divergence list
read first. The fast runner started on the base tree; sisters one level at a time with per-binary
runs; the full sweep + the final suite before the report.

Progress (16:05 UTC): all four Agent (d2) sisters GREEN on the final tree — 12 3/3, 13 3/3, 14 9/9,
15 6/6 (per-binary --test-threads=1 runs; the agent loads assert Drones::difficulty == 2 at load).
15's fight: the plank grenadiers (#7/#8) and the west-room grenadiers (#10/#11) pre-shot from
door[39] north and inside door[29] [L the d2 load probe spots]; the BodyArmourHeavy #802 detour
DROPPED — the pickup sits in the x ≈ 25 pit behind the ch_lock 0xc8 door at (25.1, 41.6, 36.6)
(the detour walk wedged airborne on it, floor_below 40.28), so the #526 rim-shot blast stays on
health: the d1 27.73 → the d2 41.60 = ×1.5 [M 0x1902c8 GameState+0x28==2 → ×Plr_DMod_Normal; the
measured ratio pins the row]. The fast runner (tools/test/suite.py) has been on the tree since
15:15; the walkreal_07000015_live FAILED line in it is the stale 15:05 binary — superseded by the
final-tree 6/6 rerun. Remaining: the suite total line, the report, the finish steps.

Correction (16:15 UTC): the earlier "the measured 27.73 → 41.60 = ×1.5 pins the Plr_DMod row"
line was wrong reasoning — the 27.73 was the older run's grenade-set blast at 1.7 u, a different
geometry from the rim shot's 2.44 u, so the ratio is not a row ratio. The rows are pinned
properly instead: [L ee_start_05.bin @0x30cc50/54/58 (ELF symtab Plr_DMod_Easy/Normal/Hard) =
0.6/0.7/1.0] matches the port's damage_factor 0x05..=0x08 group (0.6, 0.7, 1.0) [M ReadTuningVars
0x1d3a80's jump table]; 07000015 runs the 0x14..=0x16 group (0.4, 0.6, 1.0). The test comment is
fixed in the same push.

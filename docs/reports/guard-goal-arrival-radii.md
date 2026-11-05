# guard-goal-arrival-radii — guards stop at each goal's own arrival radius

Branch `s/guard-goal-arrival-radii` = origin/master 2f0c88e2c + `origin/s/level-12-engage-range` merged
(3593c8cf2). The level-12 session's reverted fix is re-landed as form 1 with the census the report asked
for; the cross-level walk cost is now MEASURED and is the honest headline: **7 red tests across 5
binaries** (6 walk pins + 1 ai pin) that the next session must re-route — the analysis and the first
re-route moves are on the branch.

## Landed

**Form 1 — the per-goal arrival radius** (commit 6301381e5). `Drone.goal_arrival` (the port's
`route+0x8C = Drone+0x8EC` [M MoveToGoalPosition 0x1517d8:49-52 — arrives under it; seeded 2.0 by
DefaultInit 0x14b300:579; rewritten by SetupRouteToPosition from each goal's radius, 0 → 2.0 [M 0x158b10];
NOT touched by SetupRouteToObject [M 0x158948]]) is set by every position-goal setter to its game radius,
and `move_to_goal` arrives under it instead of the flat 2.0:

| goal | radius | evidence |
|---|---|---|
| cover (FindCover) | 0.5 for node kind 0/1, else 1.0 | [M 0x152548:174-181 — the chosen node's first byte 0/1 → 0x3f000000, else 0x3f800000] |
| ambush cover | 0.5 | [M 0x152958:207-208] |
| safety-from-explosive | 1.0 | [M 0x152250:126-127, 163-164] |
| alarm point (fresh + reuse) | the AIPoint's own +0x14 accuracy | [M 0x153008:20-23 fresh / :45-48 reuse] |
| run-to point | the AIPoint's own +0x14 accuracy | [M 0x153168:39-43] |
| mission fresh path | 1.0 | [M 0x153a30:141-142] |
| mission retry / player | 2.0 | [M 0x153a30:168-169, 205-206] |
| closer-position probe | 3.0 (saved/restored around the probe) | [M 0x155e10:30] |
| goal-to-drone-with-LOS | 3.0 | [M 0x1543f8:100-101 — MoveToGoalPosition(0x40400000, …)] |
| interrogate assist | 10.0 | [M 0x15add0 + callers] |
| sound alert route | 2.0 (explicit) | [M 0x154aa8:11-13] |
| home (SEARCH_DONE) / astronaut strafe | 2.0 (the DefaultInit seed; no [M] radius found) | [S] kept at the seed |

`alarm_point` widened to carry the reuse arm's accuracy `(yaw, pos, accuracy)` [M 0x153008:45-48 re-runs
SetupGoalPosition with the same point accuracy]. Form 2 (the persistent 0x8EC carry into
`MoveToObject`) is NOT landed — form 1 first per the task.

**The [D] accuracy census** (commit 6301381e5, `crates/nf-game/tests/aipoint_accuracy_census.rs`): every
07* level's 0xE9 placements through the port's own `AiPoint::from_placement`. Result — accuracy 1.0
everywhere except: 07000003 (1.0×5 + 2.0×4), 07000006 (1.0×5 + 3.0×1), 07000008 (2.0×1). Levels with AI
points at all: 01(11) 02(6) 03(9) 05(3) 06(6) 07(33) 08(1) 09(11) 0a(8) 0c(1) 11(12) 12(13) 13(6) 14(2)
15(1) 16(4) 4a(2). So the point goals move 2.0 → 1.0 on fifteen levels — the census predicted the blast
radius and the walks below confirm it.

## The reds (form 1's real cross-level cost, all reproduced this session)

| binary | failing test | note |
|---|---|---|
| walkreal_07000001_live | …_real_damage_00agent | the level-12 session's known 001 regression; my re-route moves (below) pushed the death tick 5540 → 19098 and kept d1/d2 green |
| walkreal_07000006_live | walk_the_castle_interior_to_the_party_real_damage_agent | new |
| walkreal_07000013_live | walk_the_warm_chain_to_the_front_entrance_and_out_real_damage_agent | new |
| walkreal_07000015_live | walk_spawn_to_the_omega_sector_exit_real_damage | new |
| walkreal_07000016_live | the_chasers_punch_lands_the_record_damage + walk_the_chasers_defeat_completes_the_mission_on_real_damage | new |
| ai_live | guard_takes_cover_like_the_game | the pinned live in-cover position (58.83, 99.32) [L] was measured under the game's own radii — with cover arrival 0.5 the port's guard stands deeper than the pin; either the live fight's goal was not FindCover's, or the pin's tolerance must widen with the address cited |

The mechanism is the report-predicted one: the true radii (1.0 points, 0.5/1.0 covers) stand the guards
~1 unit closer, and the walks' real-damage balances were tuned on the 2.0 stand-in. The 001 work shows
the shape of the fix: the d3-gated hall band 3.2 → 4.0 and the d3 hold line z 8.8 → 7.6
(walkreal_07000001_live.rs, its own fight change, not a helper rule) moved the death from the hall
entrance (tick 5540) to the stage-walk phase (19098) where guard #6 — enabled AFTER the d3 lip-clear
pass ran — ambushes Bond at 1.9 u with a 12-hp wound. The next session's move: a post-escort re-sweep of
the lip radius (the same scan, re-run once the crossing begins), then the same treatment per red walk
(006/013/015/016 need their own per-walk reviews; their fight loops and hold lines are per-test code).

## Runs this session (NF_ISO set, --test-threads=1)

| what | result |
|---|---|
| nf_game (lib) | green (724-0 across the first sweep chunk; the unit tests ride the sweep) |
| aipoint_accuracy_census [D] | 1 passed, 0 failed (values above) |
| walkreal sweep (12 of 26 walkreal binaries reached: 01-09, 0b, 0c… see below) | the reds above; the rest green |
| ai_live | 1 failed (guard_takes_cover_like_the_game), rest of the binary green |
| the sweep state | 140/1132 binaries run, 1072 passed / 8 failed / 41 ignored cumulative when the window closed; disc_golden's 3 slim-ISO failures are inside that count |

The walkreal binaries NOT reached by the sweep remain unrun (002/004/010/0d… — the sweep order put every
walkreal before the ai/door/guard set; the window cut at walkreal_07000016). The reds are all real-damage
variants — the plain walks passed wherever they ran.

## What's left

1. The 6 walk re-routes + the ai_live cover pin decision (the 001 recipe above; each red walk needs its
   own per-test fight/route change — per-level opt-in, never a shared-helper rule).
2. The remaining walkreal binaries (the sweep state file is resumable: /tmp/order.txt + the chunk runner
   pattern from the report's session).
3. Form 2 (the 0x8EC carry into MoveToObject) — only if the decompile review says MoveToObject's
   arrival must see the last position route's radius [M 0x151618:65-67 reads route+0x8C; the port's
   move_to_object arrival is its own 2.0 today].
4. The full suite re-run on the final tree (the driver's fast runner) once the walks are green.

## Live checks for the main agent

1. On 07000012: after the alarm, a guard in GoToGoal (0x63) toward an AI point — log `Drone+0x8EC` at the
   stop. Expected [M 0x153008/0x153168]: +0x8EC = the point's param-1 accuracy (1.0 on this level [D the
   census]), and the stop distance from the point is under it (the guard stands ~1 unit deeper than the
   old 2.0 rule).
2. The same for a cover goal: +0x8EC ∈ {0.5, 1.0} by the node kind byte [M 0x152548:174-181].
3. ai_live's cover fight (the 0x05 spawner template guard): the live in-cover position under the game's
   own radii — does it match (58.83, 99.32) still, or does the game's guard stand where the port now
   stands? This decides whether the pin or my radius reading is wrong.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures anywhere I
can read (no banner, no status line, no API surface exposed to the worker).

# agent-stream-join

**This branch moves the shared random stream on every level.** It joins the stream-moving branches
on master 6186d8530 for one re-pin. It is not finished: 35 walk tests are red (list below).

## Joined

stream-batch-8, agent-lvl-13, agent-lvl-03-04, agent-lvl-01-02, agent-lvl-12, agent-lvl-14,
agent-lvl-15-16, agent-lvl-1b, agent-guards, agent-glass-2, agent-lvl-0d-11, agent-lvl-09-0a,
agent-weapons, agent-lvl-0b-0c, agent-turret-hits, agent-lvl-07-08, agent-pane-solid,
agent-ally-bullets, agent-lvl-05-06-b, agent-engage-range, agent-enemy-rounds, agent-qclaw-walls,
agent-qclaw-gate, agent-music-merged, agent-07-door-guards, agent-aim-3, agent-1b-panel49,
agent-sound-release, agent-01-sight, agent-use-pick, agent-0a-door725, agent-09-lift-call.

**Not joined: agent-guard-breakables.** Its bullet pass (`Bullet::update_objs`, `BulletEvent::Object`)
and agent-ally-bullets' drone pick (`update_with`, `BulletEvent::Drone`) rewrite the same function,
and the lvl-12 pane drain already covers part of it. The two need one combined contact walk.

## Code fixes made here

- The spawn draws the miss wobble `Rand_Rand(10000)` before the sight constants [M 0x137cb0:58-59,
  :81 → 0x14d890]. The 07000011 engage ranges are 5.0/6.0, on load and spawn alike [M 0x14b300 LAB_0014bb44].
- The explosive scan takes its gates: `+0x2B` clear, `ObjectVisibility` >= 0.7 (cone, 50-unit cap),
  and the head ray [M 0x1435f0:18-44, 0x1781a8].
- AttackTalk cases 5/6 draw inside the window on the unseen counter `+0x274` (not the seen
  counter), and case 7 draws on 14-16 [M 0x13eac0:73-90, 0x176a28:198-221].
- Guard sight: another drone's body on the ray blocks it. `Collide_LineOfSight` picks every object but
  the looker and the target, and the 0xa27 flags keep the class-2 drones [M 0x1ec690, 0x1eae80:57-85,
  0x1ebaa8:28-31 → Collide_Jointy]. The body test is the posed bone boxes [S].
- `NF_AILOG` is read once, not on every per-guard tick.
- The merges' code conflicts are reconciled on one path each: one spawner slot walk (lvl-13's, with
  0d-11's template records and extra guards), one level-sight helper, one script-object triangle
  stage (lvl-1b), one turret part test (turret-hits), one aim snap (aim-3, which also passes
  panel49's live pin), and one claw latch (qclaw-walls' walled bolt behind qclaw-gate's icon gate).

## Test changes

- The claw stances on 07000014 and 0700000d are moved into the type-2 icon arms where the walled
  bolt reaches the point. The 14 trench grating's slats (z 42.39 and 43.39) take the bolt.
- `use_pick_census_live` restores the mission per press. `arrival_plates_06_live` adds the
  once-per-load pin [L live-1108 §3]. `guard_sight_bodies_live` is new. The MP fixed-run digest is
  re-pinned (two equal runs).

## Results on f1fb1e1ed (one test per process, real ISO)

Green: every probe binary, 06, 09, 0c_barrelrow, 0c_eastroute, 0d, 11_route, 12, 13 (warm chain still
ignored as on master), 41, 4a, 4a_satchel, load_scene_digest, load_scene_equality, tick_cost,
mp_perf_16, level12_bridge, assassin_04, death_channel_timing, music_unported_consumers, audit_04,
interactables_15.

Red (35):

| binary | test | measured end |
|---|---|---|
| 01 | spawn_to_exit real_damage, _agent | meet plate ch 0x32 not raised |
| 02 | castle real_health_agent | Bond died t11853 |
| 03 | full_route real_damage | Bond died t414860 |
| 04 | walk_the_full_mission + the 3 stats/outro tests | the target lives (the walk runs dry on her) |
| 05 | door_band_exit real_damage_00agent | wedged on the battlement east leg |
| 07 | party real_damage_00agent | Bond died t7677 |
| 08 | whole_mission real_damage | Bond died t26394 |
| 0a | 6 office-floor walks | an alarm runner (team 3 tower guard) is not caught; the walk's clear only targets team 1 |
| 0b | balcony_to_roof, fresh_spawn_to_end | wedged at door #3's approach (9.59, 50.85, 19.77) |
| 0c | whole_mission real_damage | the camera is not held at the photo |
| 11 | lower_floor real_damage (d1, agent, 00agent) | caps on the ledge west / spawn pocket; one death t2033 |
| 14 | e_use_switch, infiltration, tripbomb | walk_to the 1.72 landing cap |
| 15 | omega exit (plain, agent, real_damage) | fence #447 goes by bullet (cause 2), not the blast |
| 16 | chasers_punch | Bond died t1890 |
| 1b | drakes_defeat | the missiles take the disarm branch |
| hostage_save_02 | saved geisha stands | she does not stand |
| interactables_1b | pit klaxons | four triggers leave the list |
| spawners_13 | lift squad | guard #0's death channel 0x2A not raised |

None of these is traced to a cause yet, except 0a, where the weapon path isn't traced: the runner is
team 3 and the walk never shoots team 3.

## Unproven

- The body blocking of guard sight is read from the decompile. The port's foot ray still clears a
  contact standing on the line, which is where the 01 contact is shot by the start guard (ally
  rounds now land). Whether the game's start guard kills the contact for an idle Bond needs a live look.
- walkreal_07000013's warm-chain real_damage walk stays ignored; the re-pin was not reached.

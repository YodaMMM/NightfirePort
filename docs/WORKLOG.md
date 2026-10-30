
## level-14-door-guards-combat

- (14:35 UTC, 10-30) Branch created from origin/master (a19be699). Task: on 07000014 after
  the fusebox release (ch 0x87 + ch 0x88 -> 0x8C) with Bond held at (-76.8, -1.1, 46.5),
  the game walks two guards in ENEMY_MISSION (0x1b, sub 0x06) to the west door and stops
  them there; the port instead raises alert to 1.00 with no los/seen and flips drones to
  COMBAT_OUT_OF_RANGE (0x5a) and then OPEN_DOOR (0x81) re-pressing the locked door. Find
  the port reader that raises alert/enters combat without sight, fix it with [M] cites
  (ENEMY_MISSION DSTATE, alert writers, 0x8C release path; [D] placement params), add the
  repro test (3000 ticks, no drone in 0x81/0x5a, the two 0x1b guards stop by the door).
  Repo/ISO/work/target kept from the guard-opponent-pointer-2 session; suite runner to
  start in the background next.

- (16:05 UTC, 10-30) Checkpoint: the repro is on the branch (door_guards_hold_07000014_live,
  1 passed) and the port symptom is fully traced — the 0x1b→combat flip is the ENEMY_MISSION
  arrival path [M 0x15e1e8 case 3] firing early because Route::follow returns ARRIVED at
  radius 2.0 while the body is ~1.5 short (the game's walkers stop AT their route ends and
  never get code 3), plus the group-9 clones spawning at alert 1.0 (template prop decode
  unsettled). The 0x96 alarm OR#323 fires but carries no msg at the flip. Report:
  docs/reports/level-14-door-guards-combat.md. Fix not landed; next session continues.

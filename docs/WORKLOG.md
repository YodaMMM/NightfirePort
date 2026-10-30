
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

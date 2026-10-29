# WORKLOG

## level-05-every-objective-2 (started 22:12 UTC 2026-10-29)

Task: The Exchange castle (07000005) — un-ignore `walk_the_west_ledge_and_cross_the_cable`
inside `walkreal_07000005_live`: walk the west ledge, jump-grab the cable at the icons, ride
it hand over hand to the far end and drop off, on real input (no teleport/poke), then continue
the mission. Fix any port gap the real leg hits [M/D]. FULL suite + total line. Branch
`s/level-05-every-objective-2`, from origin/master 531c08fb.

Progress log:
- 22:09 setup checked (repo, work/, ISO, target-cache, nf-queue all present); branch created and pushed with this entry.
- 22:10 fast suite runner launched in the background (tools/test/suite.py, NF_ISO set, PKG_CONFIG_PATH + CARGO_TARGET_DIR exported; cargo needed the full PATH in the detached env).
- 22:12 read master's walkreal_07000005_live.rs + the level-05-every-objective and level-05-zip-door reports: the un-ignore + the west lane + the notch grab + the strafe ride + the drop + the south legs are ALREADY on master (9/0/0 green, west-lane-3..8 + zip-door). The one remaining gap on the route: the band's north seal at z −20.5 (zip-door's pinned blocker) — "then continue the mission" hangs on it.
- 22:18 probe_seal05.rs written (the seal region x −36..−20, z −14..−27: floors + headroom + six 0.25-resolution cut lanes) — queued behind the suite build for the cargo lock.
- 22:31 suite build done (908 binaries, 1142 s); the run phase continues.
- 22:20-22:45 THE SEAL DECODED OFF-ROUTE [D probe_sealtris05 + probe_sealwalk05 + probe_ledgewalk05]:
  the band's north notches are pillar-closed (0.12-0.38 clear vs the 1.1 capsule) — but the
  route never needed them: the WINDOW LEDGE (x −35, disc floors 36.0-36.43 running z −19..−25.5)
  walks clean from the approach's west end to the crawl prompt #85 zone (5 legs, plain forward,
  no falls), and the E press (yaw π along the #35 axis, edge per 6) enters the wall-hug window
  crawl (SS_CREEP snapped to (−34.10, −26.29)), which crawls south 490 ticks and exits at
  (−34.38, −36.07) — the band's south end. No port gap: master's collision/crawl carried every leg.
- 22:50 walkreal_07000005_live::the_cable_landing_walks_the_south_walkway_toward_the_zip_door
  extended: the seal negative kept (its lanes are [D]-sealed), then the ledge legs + the #85 E
  press + the crawl south with [M/D/L] cites; two walk_to::Use attempts refused (the act faces
  the target → north, off the crawl axis) — the raw yaw-π press is the game's shape. 9/9 green
  (94.9 s). The corridor south of the crawl exit wedged the probe's steer — next session's leg.
- 23:15 pushing the checkpoint; the report + finish steps follow; the suite total lands when the
  runner finishes.

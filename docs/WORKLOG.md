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

# WORKLOG

## level-05-terrace-walk (branch s/level-05-terrace-walk)

- start (master d1b31fd3). Task: 07000005 — `walkreal_07000005_live` still places Bond at
  [22.0, 50.1, -68.0] on the terrace (floor 49.001 [D probe_terrace05 col (21.8, -68.1)]) as a
  stand-in (the comment above that write, ~line 1365); find the game's route from where the
  test's walk stands to that terrace spot [D collision/placements incl. doors, ladders, wires,
  grapple points; M the classes the route uses], walk it on real input, delete the stand-in
  write; if the data shows no route, pin that with a sister assert and name the live check
  that settles it. Run walkreal_07000005_live and interactables_07000005_live; quote totals.
- outcome (2026-10-31, sandbox deadline 14:24 UTC): THE TERRACE ROUTE IS WALKED — the stand-in
  placement write is deleted. The disc's own continuation past the covered staircase's top [D
  probe_terroute05 maps A-F over the port's collision of the disc's data]: top flight → the south
  strip (43.9-44.1, z −72.25..−73.5) → north over the walkway tip (x ~43.5) → the east walkway
  (44.0-45.2, x 38-49 — the door guards #271/#272 post on it) → the west staircase (45.0→46.6 at
  z −59.5, 47.0 band, 47.4→48.8 down z −61.5..−64.5) → the y-49 slab at z −65.5 → through the
  DOORHOUSE DOORWAY (the west wall's two planes x 37.0/x 37.5 each gap at z −67.75..−69.25 [D the
  y-50.2 wall sections]) → the terrace spot (22.0, −68.0), floor 49.001 [D probe_terrace05]. The
  blocker that parked the east legs on 2026-10-29 was the TEST's own drop steering (the aim-at-(0,−72)
  release pulled Bond off the lane into a PlungeTrigger → the death NIS SP #284 disabled the sim +
  wrote the mission-fail ch 0x62 [L the NF_SCRIPTLOG run]); the straight release drop lands on the
  lane and no script window outlasts the hold — the PARKED early-return is gone. The old "no
  walkable link" door-test claim was the scan's x ≤ 38 window (the TEST comment was wrong, not the
  disc). the_door_band_exit_chains_to_07000006_walked: walked cold start → … → the spot → the arch
  E press → ch 0x1E → level_exit 0x07000006. walkreal_07000005_live 11 passed/0 failed (98.82 s on
  the final tree), interactables_07000005_live 3/0 (6.38 s). Final tree = origin/master bf6b5a2a
  merged. Full suite on the final tree, chunked to the sandbox deadline: 431 passed / 0 failed /
  22 ignored across two chunks (421 s + 270 s of ~986 binaries — the remainder did not fit the
  window; my two touched binaries ran green directly). Report:
  docs/reports/level-05-terrace-walk.md (3 live checks listed for the main agent).

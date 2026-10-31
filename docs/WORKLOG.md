
## level-09-alarm-switch-walk — started (in progress)

- 10:57 UTC branch s/level-09-alarm-switch-walk created from origin/master b44ac857 (sandbox
  warm from the level-14 session: repo, work/, ISO, rust, target in place). NEXT: read
  walkreal_07000009_live.rs the_alarm_switch_fails_objective_1 (~:248) and the file's existing
  Walker legs, replace the sim.bond.pos teleport beside the ch-124 switch with a real walk,
  keep the aim + E + the ch 0x7C -> mux #278 -> ch 0x61 "Alarm Raised" chain [D mux #278,
  M 0x1827d8:128-131] assertions, fix any port-side blocker, run the 09 binaries.

## level-09-alarm-switch-walk — done

- 11:12 the walk landed: the_alarm_switch_fails_objective_1 walks the ch-124 switch (the port's
  list order makes that the LOBBY switch at (18.74,-368.05,43.17), not the car-park #253) —
  spawn -> stair base -> lobby (the walked test's legs, patrol-beat clears) -> a NO-CLEAR nav
  leg to the switch (clear_live_guards chases ever_seen guards across floors: two walked runs
  dragged Bond down to the car park mid-leg and a dxz-only arrive then "reached" the switch
  from 8 units below) -> direct try_walk_to 0.95 -> same-floor |dy|<2 + dxz<1.4 asserts ->
  the UNCHANGED aim + E loop fires ch 0x7C -> mux #278 -> ch 0x61 -> the fail objective
  [D mux #278, M 0x1827d8:128-131]. No teleport, no pokes, no sim-code change (the two route
  behaviours are the helpers' design, documented in the test).
- 11:20 runs: walkreal_07000009_live 4/0; every *07000009* binary (5) 27 passed/0 failed/
  2 ignored. Suite chunks 1-5 on the warm target: 203/984 binaries, 695 passed/0 failed/
  71 ignored; fix_0c_pylon_live rc 101 (headless render [S]) and mp_online_failures rc -9
  (sandbox SIGKILL at startup) are environment, not tree; the remaining 781 need hours —
  resume command in the report.
- 11:55 the report: docs/reports/level-09-alarm-switch-walk.md.

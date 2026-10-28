# WORKLOG

## level-05-every-objective (2026-10-28, branch s/level-05-every-objective)

Task: make crates/nf-view/tests/walkreal_07000005_live.rs walk the WHOLE
mission 07000005 from a fresh load on real input (step_bond only; no teleports
past a blocker, no channel or state pokes): every objective the level's mission
data lists [D the objective records / Mission_MonitorObjectives 0x1827d8 M],
in the order a player does them, each asserted complete, ending on the level
exit firing. First list which objectives the current tests already cover; add
legs for the rest. Every blocker is a port gap: find the game's behaviour
[M/D] and fix it, or list it for the main agent if drawing only.

Plan:
- Fast runner started in the background in the first minutes.
- [D] list 07000005's objective records; [M] read Mission_MonitorObjectives
  0x1827d8 and how each objective type completes.
- Audit walkreal_07000005_live.rs coverage; add legs for the uncovered
  objectives in player order.
- Fix port gaps found on the way; drawing-only gaps listed for the main agent.
- Full suite on the final tree; push; report; finish steps.

## level-05-every-objective — progress (2026-10-28 ~18:30 UTC)

- [D] the objective records dumped from ACTION.ELF .data (scripts/dump_mission_table.py):
  mission 0x07000005 = 6 records over 4 parts (05:1, 06:1, 07:2, 08:2); level 07000005's own
  record is the single "Breach the castle walls" (cond 0x46, enable 0x45) — the port's
  mission/c07000005.rs parse is exact.
- Coverage audit: obj 0 is covered end to end by walkreal's truck test (announce [L snapshot],
  complete at the 06 arrival [D OR #498], the ch-0x1D exit); the cable route's legs stop at the
  notch ride — the ride → zip door #112 exit legs never existed.
- New leg test the_cable_landing_walks_the_south_walkway_toward_the_zip_door: the full cable
  route (slab, gap, notch grab, strafe ride) then the south walk. PINS the blocker [D]:
  the landing's south rail (z ~0) is held by the disc's own trench (z −12, floor 23.5) and the
  void band z −40..−64 (probe_south05: no floor at any x below y 55); the door's battlement
  band (z −72, the game's nav chain #149-155, guards #271/#272) does not connect in the disc's
  data. Wire #251 is only the 8-unit hand-over-hand cable [D its bounds].
- Suite runner restarted with the ALSA prefix fixed (/tmp was wiped; scripts/run_suite.sh).

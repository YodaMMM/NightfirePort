# Worklog

## level-05-zip-door (branch s/level-05-zip-door, from origin/master ad2f0791)

Sandbox opened 20:08 UTC, expires ~22:06 UTC (stop new work 21:46). Scope: get The Exchange
Part 1's cable route from the cable landing to zip door #112 [D walkway/door/trigger placements
+ collision; M the door's use/approach path and any scripted carry], turn walkreal_07000005_live's
pinned negative into the positive leg on real input through the zip line; then 07000006's OPEN
LEG — the upper gallery -> dressing-room LoadPoint #746 walked route [D nav, placements]. Full
suite + total line before the final push.

Setup: sandbox had stale leftovers from an earlier session (empty NightfirePort dir, work-slim.7z,
alsausr prefix, nf-queue worktree dir); re-cloned fresh from origin/master ad2f0791. ISO download
starting next (foreground).

Plan: (1) read walkreal_07000005_live.rs + done/10-level-05-every-objective.md for the pinned
blocker evidence; (2) sweep placements/triggers/collision around the cable landing [D];
(3) find the game's approach path to door #112 [M Player_Activate/use path]; (4) port the gap,
walk the leg; (5) 07000006 gallery->dressing-room route; (6) suite + report.

Progress (21:15 UTC): the route to the zip door is DECODED from the disc's own data — the old
"south structures stop" read was a scan blind spot (the old probe covered x −26..50; the window
ledge and the wall-hug band live at x −35..−33). The full chain [D probe_southwest05's band maps
+ the placement census]: landing → the south piece past the L-rail → a 1.5-unit CRACK hop at
z −11.5..−12.5 → the approach (z −16..−19) → the wall-hug band at y 36.5-37.6 (ai point #259
36.5, the guards #65/#68 feet 37.6) → the window crawl #35 (prompts #84/#85, plates #311-#314) →
the roofed corridor → a second crack hop at z −46..−50 → the south gallery → the SW loop (the
west walkway x −41..−40, the z −78 band, the z −76 spine) → the west battlement (plate #269
fires the copter) → THE MOAT INLET (x −15..−5, floors 20-23) crossed by the Q-CLAW grapple to
pulley #281 ((−3.40, 42.94, −71.77), 17.6 units, inside the bolt's 50 [M 0x12e7b0:158]) → the
east battlement → the rising stairs → the y-49 door band → door #112 E → 0x1E → 07000006.
WALKED GREEN on real input: the drop, leg 1 (the L-rail lane), the crack hop (+20 ticks), the
approach to z −19.6. PINNED: the band's north seal at z −20.5 (four lanes walled/roofed — the
rim at y 38 has 1.6 headroom vs Bond's 1.86 body). Also found: the port's plunge shell #291's
sheet passes through y 30.7 at z −10.4 — a 0.4-unit lip-dip fires ch 0xBE (the game's wall-hug
route runs the same ledge safely [D the route's own plates] — the live check decides whose
contact rule is coarse). walkreal_07000005_live: 9 passed / 0 failed (69.5 s). The fast suite
daemon restarted 21:09 on the final tree (the first launch raced a mid-patch build; BUILD FAILED
is that race, not the tree). Remaining: the seal's game-side answer + the un-walked legs; the 06
gallery probe (probe_gallery06.rs committed) timed out under suite contention — next session.

## level-0d-every-objective (started 2026-10-28)

- Branch s/level-0d-every-objective opened from origin/master 284c1676 (fresh sandbox,
  full setup redone; WORKLOG restored from origin/s/level-05-zip-door — master carries
  no docs/WORKLOG.md). Task: walk the WHOLE 0700000d mission from a fresh load on real
  input in crates/nf-view/tests/walkreal_0700000d_live.rs — every objective the mission
  data lists [D records / Mission_MonitorObjectives 0x1827d8 M], in player order, each
  asserted complete, ending on the level exit firing; blockers are port gaps to fix with
  [M/D] evidence or list for the main agent if draw-only.

- level-0d-every-objective: the walkreal_0700000d_live walk EXTENDED TO THE END. The old
  documented stop at the C5 door (objective 6 uncompletable) was obsolete: the level census
  (docs/levels/0700000d.md §7, 2026-10-22) corrected §6 — ch 0x1f IS reachable by shooting
  (`Destroy_CollisionHandler` 0x12bd48 subtracts every class-5 bullet's damage, death writes
  switch_channels [M :35]; the disc hint "considerable firepower" 0x4000066 [D]). New legs:
  the shooting stand 11+ m off the bigdoor02 #96 yard face (blast-reach guard, top_up_health),
  the mounted laser 0x67 [D GunImp #225; L probe clip 0/spare 0 sustains fire, 50 dmg/hit],
  shoot_at through the real trigger path until ch 0x1f (~200 hits on health 10000 [D]),
  objective 6 asserted complete (states[12][2] == 4 [M 0x1827d8:88-99]), the corridor walk
  through the dead door raises ch 0x28 (the re-firing Touch #433 [D; M triggers.rs]), the AND
  #434 [0x1e & 0x1f & 0x28] raises ch 100, `Mission_Update` state 6 [M 0x182c60] ends to
  0x730000d [D record] — asserted. Every objective the mission data lists now asserted
  complete: 3 (record 11, start trigger ch 0x23), 4 (0x1d), 5 (0x1e), 6 (0x1f). The remaining
  stand-in: the crane sequence into the yard (SP #115 script 0x060009EF ride) — documented,
  unchanged.
- Run (this sandbox: background jobs die between tool calls; chunk_suite.py + targeted runs):
  walkreal_0700000d_live 1/0 (5.1 s), interactables_0700000d_live 11/0, walk_0700000d_live
  3/0, patrol_walk_0c_0d_live 2/0, census/audit_0700000d 2/0, gunimp 0d family 6/0,
  nf-view crate sweep (timeout-cut at 9.3 min) 179/0 across 38 binaries, chunk_suite
  cumulative 14 binaries 258/0 recorded (885-bin full sweep cannot fit the 2 h window —
  third sandbox to measure the background-kill + build cost).

## level-02-every-objective (started 2026-10-28)

- Branch s/level-02-every-objective opened from origin/master 87256135. Sandbox carries the
  level-0c session's setup (repo + work/ + Nightfire-onfoot.iso present; skipped re-download).
  Task: walk the WHOLE 07000002 mission from a fresh load on real input in
  crates/nf-view/tests/walkreal_07000002_live.rs — every objective the mission data lists
  [D records / Mission_MonitorObjectives 0x1827d8 M], in player order, each asserted complete,
  ending on the level exit firing; first list what the current test already covers, then add
  legs for the rest; blockers are port gaps to fix with [M/D] evidence or list for the main
  agent if draw-only.
- Plan: (1) read walkreal_07000002_live.rs + docs/levels/07000002{,-audit}.md for covered
  objectives; (2) decode the level's objective records [D] + Mission_MonitorObjectives [M];
  (3) add walk legs for uncovered objectives; (4) full suite (fast runner started 22:52 UTC),
  total line before push.

## level-03-every-objective (started 2026-10-28)

- Branch s/level-03-every-objective opened from origin/master 87256135 (fresh sandbox;
  WORKLOG restored from origin/s/level-02-every-objective — master carries no
  docs/WORKLOG.md). Task: walk the WHOLE 07000003 mission from a fresh load on real input
  in crates/nf-view/tests/walkreal_07000003_live.rs — every objective the mission data
  lists [D records / Mission_MonitorObjectives 0x1827d8 M], in player order, each
  asserted complete, ending on the level exit firing; first list what the current test
  already covers, then add legs for the rest; blockers are port gaps to fix with [M/D]
  evidence or list for the main agent if draw-only.

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

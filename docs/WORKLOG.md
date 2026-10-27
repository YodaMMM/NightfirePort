# WORKLOG

One entry per task, appended at the end. Never rewrite or drop another task's entry.

## level-01-mayhew-wait

Task: on 07000001 (Double Cross Part 1) the Mayhew escort runs far ahead of Bond and goes
through doors before him; in the game the escort waits at its hold points. Find the escort
wait/hold rule in the decompile (escort follow states + player-distance gates, door-wait
logic; `escort_follow_07000001_live` and `fix_npc_doors_live` pin parts of it), the route
points on the corridor [D], fix what lets him run ahead, pin with a real-input walk test
(Bond stays back, Mayhew holds at his wait point).

Status: started 2026-10-27 ~00:52 UTC. Setup: fresh clone (sandbox reset), branch
`s/level-01-mayhew-wait` created from origin/master @ ec917093 and pushed.

### level-01-mayhew-wait — outcome (2026-10-27 ~02:20 UTC)

The ally-wait hold is implemented (Drones::tick parks the escort in 0x21 at the 0x20 node until
ch 0x99) and the release chain is proven end-to-end through the real input path: plate 0x32 →
OUT mux 0x33 → SP #276 → 06000086 → ch 0x0e → the three ambush guards → death channels →
AND mux → 0x99 → the hold releases. Green: mayhew_hold_live (new), escort_follow_07000001_live,
fix_npc_doors_live (its door-#1 premise fixed per the hold; the escort's first self-opened door
is #2). The old "ch 0x21 writer unknown" note was a mis-read of SP #276's out param (0x33).
Left: the full suite on the merged tree (this sandbox kills background processes; the suite
needs ~40 min of foreground windows), and walkreal_07000001_live may need the same
plate-crossing harness step. Report: docs/reports/level-01-mayhew-wait.md.

### level-01-mayhew-wait-2 — start (2026-10-27 ~05:58 UTC)

Task: after the ally-wait hold (node 33, flag 0x20, release ch 0x99), two full-suite tests
went red: walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact and
fix_mission_live::the_07000001_exit_works_end_to_end — both walks reach the exit without
clearing the ambush, so the contact stays parked at node 33. Plan: keep the hold; make the
walks clear the ambush on real input (the three ch-0x0e guards), then re-verify the exit
AND-touch fires with Bond + contact. Must stay green: escort_follow_07000001_live,
mayhew_hold_live, fix_npc_doors_live. Branch `s/level-01-mayhew-wait-2` from
origin/s/level-01-mayhew-wait @ 14769917, merging origin/master.

### level-01-mayhew-wait-2 — the walks clear the ambush (2026-10-27 ~07:05 UTC)

Both red tests fixed by making the walks do what the hold requires (the hold stays): the
release is ch 0x99 — the AND of the three ambush guards' death channels 0x13/0x11/0x12
[D mux pl 318] — and the game's own trace parks the contact in 0x21 at (2.94,7.22) for 1262
frames while the ambush runs [L tests/traces/01-doors.trace], so the PLAYER walks ahead into
the ambush. Root cause of the red: the meet scene's weapon hand leaves clip 32 spare 0, the
hall dodge fights run dry (guard 6 at +634 ticks), and the resupply/approach nav from inside
the hall wedges on the hall's south wall (segment t≈0.095/0.47, 3000-tick caps) — the ambush
stayed alive, ch 0x99 never rose, the contact held at node 33 and the exit died with it.
- walker.rs: appended hall_restage (meet-room corridor → east-door stage, proven legs) and
  hall_ammo_ladder (out the east door (7.9,5.3,11.1) → corridor box → back in). Additive only.
- walkreal_07000001_live: pre-hall top-up (< 48 rounds) + per-guard re-stage when the
  approach starts > 9 away + the per-guard dry ladder. PASS (level_exit to 07000002, ch 153
  on at tick 54829, contact at the AND-touch, 14 self-recovering lenient wedges).
- fix_mission_live: new ambush phase before the ride loop — Bond walks the meet plate (the
  TOUCH_ONCE sweep), waits ch 0x0e, kills the three named guards through the real fire path
  (shield on the contact), waits ch 0x99, then the escort ride runs as before. PASS (ch 0x99
  → 0x1d release → Mayhew walks 121 u → ch10 → exit end to end).
- Verified green: mayhew_hold_live 5.7s, escort_follow_07000001_live 23.0s, fix_npc_doors_live
  308.6s. Full suite next.

### level-01-mayhew-wait-2 — done (2026-10-27 ~07:25 UTC)

Both red tests green on the final tree: walkreal_07000001_live 3/3 (61.1 s), fix_mission_live
4/4 (7.2 s); mayhew_hold_live 5.7 s, escort_follow_07000001_live 23.0 s, fix_npc_doors_live
308.6 s — the driver's five, plus every nf-view test binary compiling (--no-run, 0 errors).
The full 4-crate runner could not complete in this sandbox (background processes are killed
between tool calls — nohup and setsid both confirmed dead twice); the report lists exactly
what ran and the left-over command. Report: docs/reports/level-01-mayhew-wait-2.md.

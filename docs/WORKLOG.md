# WORKLOG

One section per task, appended at the end. Never rewrite or drop another task's entry.

### level-04-audit (started 22:50 UTC, 2026-10-25)

Branch from origin/master (b312d959). Task: the 07000004 behaviour audit — every placement on
the main map [D] (characters 0x0f, doors, triggers, scripts/SPs, switches, monitors/sensors,
pickups, destroyables, breakables, ladders, grapple points, lifts, special objects), every
script stream's ops/camera streams/hand-backs, and the mission records, each row: the game's
creator/update/handlers [M], what the port does, a GAP column → docs/levels/07000004-audit.md.
Small gaps fixed directly [M] with a test each; large gaps become report sections for their own
tasks.

Plan: census the level's placements/scripts/records from the disc data; read the port's
07000004 paths (load/scene/mission + the walk test); build the audit table row by row; fix the
small gaps; suite; report.

### level-04-audit (finish log, 23:55 UTC 2026-10-25)

- The audit: docs/levels/07000004-audit.md — all 70 main-map placements [D] per-row (creator in
  0x1d04f0's dispatch [M], update/handlers, the port's status, GAP), the 9 archive scripts'
  streams decoded (the two level NISes stream-by-stream incl. the camera keys, the sound lines,
  the 0xb weapon bytes, the hand-backs; the 7 shared blast scripts mapped to the port's blast
  path), the mission record (one objective, watch 0x54, reveal 0x53, no fail channel).
- GAP-1 found + fixed (e4127e84): Script_SoundStart 0x1d8820 clears the stream's sound handle at
  its top before the gate [M 0x1d8820:19]; the port gated on sound_id.is_none() without the
  clear — a stream's second line never started (04's story stream 15: 0x5e @298 then 0x134 @654
  [D] — the 0x134 dialogue was dead). Test audit_07000004_live pins every line's id + tick
  window through the real path; the sound/cut-scene/04 neighborhood re-run green (7 binaries).
- Gaps documented for own tasks: GAP-2 the RB lamps (0x1d6bf0 machine, unported, nothing draws),
  GAP-3 the tree sway-anim runtime (draw-only), GAP-4 the camera hand-back (ledger; renders
  listed).
- Suite: the resumable suite.py clone in foreground chunks (nohup killed again) — 223/809 at
  report time, 1469 passed / 4 failed (the documented slim-ISO disc family) / 35 ignored;
  nf-game 89/89 green (the fix's crate), nf-formats 26/26, nf-audio 4/4, nf-view 105/690.
- Report docs/reports/level-04-audit.md; finish.sh + ntfy next.

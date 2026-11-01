# WORKLOG

## level-0c-route-teleports (branch s/level-0c-route-teleports)

- start (master e79794da). Task: walkreal_0700000c_live's full walk teleports Bond three times —
  out of a shaft he can drop into but not climb out of, onto a mezzanine reached (as far as the
  tests show) only by the Q-Claw from the warehouse, and onto an east deck no collision connects
  to. For each: find the game's route [D collision + placements: ladders, grapple points, lifts,
  doors; M the classes], fix any port gap with the address cited, walk it on real input; a
  grapple route drives the real Q-Claw. Delete each replaced teleport; keep the rest with a
  sister assert and name the live check. Run walkreal_0700000c_live,
  walkreal_0700000c_eastroute_live, interactables_0700000c_live and nf-game; quote the totals.

## Outcome (2026-11-01, ~01:30 UTC)

- DONE: the task's placement checklist (ladders, grapple points, lifts, doors) run on the disc's
  own data for all three walkreal_0700000c stand-ins, and the Q-Claw route IMPLEMENTED and
  driven for the mezzanine before the disc's collision closed it (the claw walk arrived at
  (13.78, 7.40, −142.10); the pick ray blocked at (15.56, 8.25, −142.21); the segment_hit scans
  then mapped the east wall solid y 6.5..15.5+ at x 14.8..15.6, no z gap −136..−150). The five
  grapple points dumped [D #308/#346/#500/#508/#511]; the Q-Claw 0x50's range 50.0 [L
  ee_start_05 weapon_data+0x1c; M Bullet_update 0x12e7b0:158]. Results: the shaft's nearest gp
  ~80 m (no claw route out); gp0 in range of the display ground's east edge but the wall stands
  in the ray, the hall lane 55.5 m out of range, the corridor's north end circular; gp3 44 m in
  range of the crate field but a ~49 m rope-pinned hang drift [M 0x192860 tail] is no game
  mechanism and the drop would be ~11.7 m. NO port gap found — every closure is the disc's own
  data. All three stand-ins KEPT with their sisters, each hardened this session; the three live
  checks named in the report (the shaft's post-drop flow; the warehouse interior entrance; any
  east-deck dynamic).
- Named binaries (final tree): walkreal_0700000c_live 2 passed / 0 failed / 0 ignored;
  walkreal_0700000c_eastroute_live 1 / 0 / 0; interactables_0700000c_live 5 / 0 / 0; nf_game
  594 / 0 / 5.
- Suite (tools/test/chunk_par_tmp.py, fresh state, 2 workers — the sandbox kills true
  background jobs; priorities: the 0700000c family + nf_game): 367/996 binaries recorded by the
  01:26 UTC stop line = 1704 passed / 1 failed / 103 ignored; the 1 is probe_pylon_render (the
  no-GPU wgpu panic, environment). The 0700000c family all green in-suite.

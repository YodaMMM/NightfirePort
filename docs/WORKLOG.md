# WORKLOG

## aim-stop-08 (2026-11-13T02:21Z, branch s/aim-stop-08)

- Claimed. Base: origin/master (23b0ae649). Task: add 07000008 to the zero_root_live list
  (the AimStandFire root stop, Drone::zero_root, Drone+0x38 [M 0x166328:117-123 ->
  DroneAnim_PreTransCallback 0x13b490]) and re-tune the level's walks against the stop on
  real input (no lowered damage/health, no top-ups, helpers gated to this level); then
  walkreal_07000008_live and the full suite via tools/test/suite.py.
- Sandbox was reset (fresh clone at 02:21 UTC, expiry ~04:20 UTC). Setup in progress.
- Done. The stop live on 08 (the zero_root_live list); the Operative real-damage walk green
  (the secure room answered from the armour corner + the per-bolt 10-u floor answer, real
  input only, !tops_up-gated); the Agent walk answered (armour corner + the d2-gated 60 u/60
  tick dodge) and IGNORED with its ledger (dies at the Hind, 41.9-health arrival, the
  machine's −38.91 rocket blast + drips) — the route's health budget re-plan and the machine
  fight's forecast launch gate are the next levers. walkreal_07000008_live 4 passed 0 failed
  2 ignored; hurt/walk/interactables/decomp_audit/models_sweep/zipline 08 all green; nf-game
  lib 795 passed. The full suite did not run (sandbox kills background jobs between tool
  calls; the foreground four-crate build filled the disk). Report: docs/reports/aim-stop-08.md.

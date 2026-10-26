# WORKLOG

## dmode-inits-hud-panes

Started 2026-10-26 ~12:20 UTC. Branch `s/dmode-inits-hud-panes` from origin/master (7e35bd77).

Task: port or prove-equal the 28 never-mentioned `NDrone2_init_DMODE_*` guard-mode init
functions (0x14d978..), the four HUD pane creators (`HUD_CreateAmmoPane` 0x1a0ed8,
`HUD_CreateHealthPane` 0x1a0f40, `HUD_CreateInfoStatusPane` 0x1a10b8,
`HUD_CreatePickupStatusPane` 0x1a1140), and `Text_SendMessage` 0x223600. For each: find
the on-foot caller, compare with the port, port diffs with [M] addresses, add tests per
ported behaviour.

Work log:
- Setup: cloned repo, created branch, pushed initial checkpoint.
- Found prior art: docs/systems/ai-dmode-inits.md + drone.rs init_properties already prove the
  DMODE dispatch census (all 555 placements NEW, only row 0 dispatched). Remaining: per-init [M]
  summaries for the 26 still-uncited family members, a disc-table row test, HUD pane create
  audit (HealthPane 4-sprite loop + timers, InfoStatus/PickupStatus secondary slots), and
  Text_SendMessage 0x223600 (not cited anywhere in crates).
- Added: docs/systems/ai-dmode-inits.md appendix (all 33 init bodies, grouped), drone.rs family
  citation, nf-formats/tests/dmode_row_table.rs (35 disc rows pinned — Defaults is not a row fn,
  fixed after first run), nf-formats/tests/hud_pane_creators.rs (glow templates == the
  HealthPane create-loop writes; ammo/health create pointers), hud-panes.md [S]->[M] upgrades
  for the four status creates + a create-fn audit section. Both new tests pass.
- Next: Text_SendMessage 0x223600 caller census ([M] __Menu_SendMessage 0x1f1c18 type-0xc arm +
  TextBox_Update 0x2237c8 self-0x53) vs the port's generic control_msg — census test pending.

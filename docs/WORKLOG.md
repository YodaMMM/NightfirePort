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

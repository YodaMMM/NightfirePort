
## port-map-player (2026-11-07, UTC)

Branch `s/port-map-player` cut from `origin/master` (22d5836b7). Task: the player family
map — one line per `Player_*` function in the decompile, ported with the port location or
not reached with the reason, to `docs/port-map/player.md`, in the `docs/port-map/drones.md`
shape. A comment does not count as ported; find the code. Any function a real level reaches
with no port code is a gap: fix with a real-level test proved red on old code, one commit
per fix. Checkpoints pushed as the table grows.

- (02:25, 11-07) The family map landed: docs/port-map/player.md, 90 rows (86 ported, 2
  absorbed, 1 not reached), the Player_UnEquipKey MISSING fixed — the keypad law wired
  (solve equips the key, case-1 hold + state-2 phase, the dropped channel re-arms kind-0
  through Lock_ReLock and clears the ring), red-then-green on the real 07000012 path;
  lock-adjacent binaries re-run green (686 lib, 3, 1, 8, 2). The grapple rope object
  documented as the render-side MISSING with live checks.

## port-map-scripts (2026-11-07, UTC)

Branch `s/port-map-scripts` cut from `origin/master` (22d5836b7). Task: the scripts
family map — one line per `Script_*`, `SP_*`, `Control_*`, `Mission_*` function in the
decompile, ported with the port location or not reached with the reason, to
`docs/port-map/scripts.md`, in the `docs/port-map/drones.md` shape. A comment does not
count as ported; find the code. Any function a real level reaches with no port code is a
gap: fix with a real-level test proved red on old code, one commit per fix. Checkpoints
pushed as the table grows.

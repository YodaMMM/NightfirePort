
## port-map-world (2026-11-07, UTC)

Branch `s/port-map-world` cut from `origin/master` (22d5836b7). Task: the world family
map — one line per `Door_*`, `Break_*`, `Destroy_*`, `Lock*`, `PC*`, `Trigger_*`,
`Sensor*`, `Camera*` function in the decompile, ported with the port location or not
reached with the reason, to `docs/port-map/world.md`, in the `docs/port-map/drones.md`
shape. A comment does not count as ported; find the code. Any function a real level
reaches with no port code is a gap: fix with a real-level test proved red on old code,
one commit per fix. Checkpoints pushed as the table grows.

- (02:50, 11-07) The family map landed: docs/port-map/world.md, 74 rows (55 ported — 3
  folded, 17 absorbed, 2 missing). The 2 missing: Lock_ReLock — the case-2 burnt-open
  re-arm dispatch is dead on master (re_lock + update_burnt_open exist with a unit test,
  zero production callers), the exact fix ships on s/port-map-player, not duplicated to
  avoid the merge collision; Cameras_PostLoad — the 3-player divider sprite, no port path
  by single-viewer design (documented in mp/rules.rs). No source files changed; nf-game
  lib re-run green (686). Report at docs/reports/port-map-world.md; ntfy failed on all
  three fallbacks (recorded there).

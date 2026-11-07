
## port-map-world (2026-11-07, UTC)

Branch `s/port-map-world` cut from `origin/master` (22d5836b7). Task: the world family
map — one line per `Door_*`, `Break_*`, `Destroy_*`, `Lock*`, `PC*`, `Trigger_*`,
`Sensor*`, `Camera*` function in the decompile, ported with the port location or not
reached with the reason, to `docs/port-map/world.md`, in the `docs/port-map/drones.md`
shape. A comment does not count as ported; find the code. Any function a real level
reaches with no port code is a gap: fix with a real-level test proved red on old code,
one commit per fix. Checkpoints pushed as the table grows.

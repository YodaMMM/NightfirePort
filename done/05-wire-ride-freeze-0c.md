# wire-ride-freeze-0c — the wire ride can freeze mid-glide on 0700000c

From docs/reports/level-0c-yard-climb-real-health-3.md (origin/s/level-0c-yard-climb-real-health-3, "What is left"): on one path phase the ride froze byte-exact at d_B 1.52-1.61 — glide 8.12 → 1.61, then wire_anim_time stuck at 22.6, anim 0x60000b5, auto_drop frozen at 20.8, substate 6, sim.disabled false: `wire_tick` stops advancing mid-ride and Bond hangs on the wire forever. Moving an earlier sweep shifted the phase and the ride completed, so it depends on the arrival phase.

Reproduce it (the report's walk with the west-edge sweep at the LADDER TOP), find which wire_tick branch stops advancing, and port the game's rule for that case [M the wire/zipline update and its auto-drop]. A player must never be stuck on the wire.

Done = a real-input test that reproduces the old freeze phase and now finishes the ride; every walkreal_*_live green; the full suite total line.

---
claimed-by: sandbox1 at 2026-11-03T19:31Z

---
## Report (done, 2026-11-03T20:43Z)

Closed by the driver: the report is on `s/wire-ride-freeze-0c` (docs/reports/wire-ride-freeze-0c.md); the worker didn't run finish.sh.

---
## Report from the worker (blocked, 2026-11-03T20:43Z)


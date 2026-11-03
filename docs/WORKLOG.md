# WORKLOG (branch running copy)

## wire-ride-freeze-0c

Started 2026-11-03 19:40 UTC, branch `s/wire-ride-freeze-0c` from origin/master (68298698).
Task: on 0700000c the wire ride can freeze mid-glide (d_B 1.52-1.61, wire_anim_time stuck 22.6,
anim 0x60000b5, auto_drop frozen 20.8, substate 6, sim.disabled false) — `wire_tick` stops
advancing mid-ride, Bond hangs forever. Reproduce with the report's walk (west-edge sweep at the
LADDER TOP), find the wire_tick branch that stops advancing, port the game's rule [M the
wire/zipline update and its auto-drop]. Done = a real-input test that reproduces the old freeze
phase and now finishes the ride; every walkreal_*_live green; full suite total line.
Source report: docs/reports/level-0c-yard-climb-real-health-3.md (origin/s/level-0c-yard-climb-real-health-3, "What is left").

### the freeze reproduced and the root cause isolated (2026-11-03 ~20:15-21:05 UTC)

The -3 walk runs on master untouched (0c-local files only; the -3's shared-helper diffs are
OLDER than master and were NOT taken). With the west-edge sweep moved to the LADDER TOP
(the -3's measured phase), the sister walk (real damage + `with_yard_aimed_stands`) froze
BYTE-EXACT: d_B 1.61, mode 0x7, anim 0x0600_00b5 at 22.6, rate 1.2, auto_drop 20.8, sub 6,
disabled false — the ride ran out of its 2400-tick window and the drop assert failed.

The per-tick probe (the ride trace now dumps the death/net/gunimp gates per tick): t473 alive
(pos −7.4489, 12.7853, −215.2419, d_B 1.64, anim b5 at 21.4), t474 the tick RAN (anim 21.4 →
22.6, root motion moved Bond to −7.44697, 12.794478, −215.27502) and `death.active` became
TRUE, t475 frozen forever. The kill is NOT a bullet (the ledger's lowest is 17.6 — 52 bullet
hits for 82.4) and no gunimp is mounted. Root cause: the hurt-volume pass (scene.rs ~3428,
[Hurt_Update 0x17e5d0 / Player_DealWithObjHit 0x194458 class-0x27 → Player_HandlePain]) runs
EVERY tick against the rider's capsule regardless of substate; at the freeze spot a volume's
damage drove health ≤ 0, `death_tick` [Player_CheckForDeath 0x190810] started the death
sequence, and step_bond's gate (scene.rs ~3350: `!death.active && !net_dead && !gunimp`)
then SKIPS `bond.tick_ctx` every tick — wire_tick never runs again, Bond hangs forever. The
-3's "NIS/anim-script gate keyed to a volume the ride's sway phase enters" = the hurt volume
+ the death gate. The freeze is phase-dependent because only certain ride phases carry the
capsule into the volume (the passing-phase ride landed at (−8.21, 12.82, −215.66), x −8.21
vs the freeze x −7.447 — the volume does not cover the whole ride path).

Left: the offline dump of the 0700000c class-0xE4 hurt volumes (which placement covers
(−7.45, 12.8..14.6, −215.3), its gate/who/kind/damage), the game's rule for the rider
(whether the collide query builds the obj+0xd0 hit list on substate 6 [Player_CollisionHandler
0x1949a8's source + disasm]), then the fix + the proof test + the walkreal sweep + the suite.

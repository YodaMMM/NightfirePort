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

### the freeze RESOLVED: the premise is refuted — it is the death sequence, not a wire_tick stall (20:45 UTC)

The hp trace (the ride probe now prints health + the death tick): the rider entered the freeze
zone at hp 1.6 (NOT the ledger's 17.6 — the ledger samples at bullet hits; 16 hp of unhooked
damage landed during the late ride), t473 hp 0.0 (the last pain ≥ 1.6 killed him mid-ride),
t474 death.active true dtick 0, t475+ dtick 1,2,3… — the death machinery RUNS (death_tick
alive from Drones::tick [M drone.rs 0x17641's site — Player_CheckForDeath 0x190810 is
"player-side, runs during cut-scenes, not gated"]), the bond tick is skipped by the faithful
dead branch [M Player_Update 0x193600:316-319 — the dead arm runs ONLY AnimObjectUpdate +
Player_HandleDeath, no substate arm: the body freezes where it died]. The port matches the
game: a rider who dies mid-wire hangs dead until the restart prompt — never a wire_tick stall.

The kill at t473 is NOT a bullet (the ledger), not a hurt volume (0700000c has ZERO class-0xE4
placements [L the scratch dump: "0700000c hurt volumes: 0 total"]), not drowning, not fall
damage (fall_damage is None on the wire), not a gunimp — by elimination it is the BLAST pain
[M spawn_blast → explode::propagate → health.pain_generic(…, pain_type 0), drone.rs ~14039]:
a destructible popped near the gap (the guards shoot the rider the whole ride — the -3's
"the settled walkers eating the sweep+ride for ~70-96") and the blast's area pain took the
1.6 hp. The Done criterion "reproduces the old freeze phase and now finishes the ride" cannot
be met by a wire fix: the rider legitimately dies. The next session: trace WHICH prop popped
at t473 (the destroy/breaks list near the gap) and whether the game's blast reaches the ride's
hang path (the PCSX2 live check), then either the walk keeps Bond alive past the ride (the -3's
remaining ledger) or the freeze-phase test asserts the death explicitly.

### the grenade watch (probe 4): the gap box saw NO grenade all ride — the blast paths are OUT

The ride trace now prints any live grenade inside x −20..8, z −230..−200: nothing all ride.
With the props/breaks also clean near the gap, the surviving kill path is the guard
contact/melee pain (not a bullet — the ledger never sees it; the settled walkers patrol the
B-end ledge the rider glides into). Next session: log the pain call sites at the t473 tick.

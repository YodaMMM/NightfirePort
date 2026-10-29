# WORKLOG

### level-0a-every-objective-3 (started 17:57 UTC, 2026-10-29)

Task: Tower B office floor (0700000a) — port the vent crawl the game uses to reach the
Q-Worm cubicle's south side [M the crawl/creep path, the vent placements D], walk it on real
input, do the real install (replace the last stand-in), then the mission tail (the ring legs
and the hatch E) the -2 branch re-derived. Un-ignore
`walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch` when green. FULL suite.

Base: origin/master (a7eace8e). The -2 report (origin/s/level-0a-every-objective-2) leaves:
walk-reachability of a bullet-clear stand (east stand furniture-blocked, desk-north band
desk-lip-blocked, south stand blocked from the nav route end z ≈ −31.5 [D probes l0a2c]) —
docs/levels/0700000a.md says the vent shortcut is the way in; the port has no vent crawl yet.

Plan: research the game's crawl/creep path + vent placements in the decomp and on disc;
start the fast suite runner early; port the crawl; extend the walkreal test through the
vent; the real install; the tail; un-ignore; full suite on the final tree.

### level-0a-every-objective-3 (mid/final, 19:45 UTC)

The last stand-in is GONE: the prologue's install is real. The route: the diagonal wall's east
end stops at x ≈ −15 (its north face n (0.30,0,0.95) spans x −18..−15 at z −29.8..−31.5 [D probe
rays]); the column at x −13.5 runs south into the band (northward rays from z −34.4 clear at
x −13/−9 [D]); the band's floor y 1.10 holds bullet-clear stands (segment_bullet CLEAR x
−16..−17.8, z −33.5..−34.4 [D probe]); the real gadget-cycle equip (9 cycles → 0x58) and 1 real
shot from the walked south stand → ch 0x48 [M PCQWorm_Update 0x1aee20] (probe_l0a3_install, all
legs arrived; the install alone alarms nobody for 1200 ticks). The walk-back needed the ring's
per-leg alarm-runner watch (a runner's press landed mid-keyboard-leg at +342 without it).

The crawl/creep system was found ALREADY ported (SS_CREEP 7, creepwall.rs, the scene arms,
[decomp Player_Creep 0x1969d0 / Player_CollCreepWall 0x196758]); 0700000a carries NO creepwall
(0x3F), no 0x3B blocker, no type-6 icon [D audit §3] — the game's crawl cannot engage there. The
under-floor space at vent ladder #686 is an EMPTY VOID in the port's collision (every ray clear,
no floor; probe_l0a3_walk) — the vent is not the install's missing piece.

The mission test re-ignored with walked evidence: the ring's mouth crossing wedged on three
routes this round (the -2 leg inside the pillar's face; east along z −15 against the NS wall
x −10.6955 z −17.5..−11.0; the z −18.6 crossing — blocked, unprobed past that). The -2 ring had
never actually been walked. walkreal_0700000a_live: 6 passed / 0 failed / 3 ignored.

### level-0a-every-objective-4 (started 20:23 UTC, 2026-10-29)

Task: continue level-0a-every-objective-3 — the walk still wedges at the ring's mouth (three
wedge points in the ignore reason on walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch).
Find the route the game's player takes through the mouth [D the collision around the mouth; M the
player's capsule resolve; L if needed], fix any port collision gap, walk the ring legs and the
hatch E on real input, un-ignore the test. Keep it ignored only with the exact remaining blocker
and evidence. Full suite on the final tree.

Plan: read the -3 report + ignore reason; decompile the mouth collision + capsule resolve; find
the walked route; port fixes; walk the ring legs + hatch E on real input; full suite.

### level-0a-every-objective-4 (final, 22:03 UTC)

The mouth is the KEYPAD DOOR: the wall x −10.45..−10.70 seals the west offices from the atrium
lane z −15.38..−23.70 [D coll-scan + probe rays] — the only opening is door #724's doorway
(jambs z −18.04/−19.36), locked on ch 0x30 whose only writers are the kind-0 keypads #648/#732
(p1 = 1, p5 = 48 [D]) at (−10.69/−10.45, 1.52, −17.46); the game's ai graph crosses exactly
there (pocket → (−11.01,−18.60) → (−9.88,−18.62) → node 0 [D]). Ported the keypad E-path's sim
side (the audit's GAP-1 machine): Lock.facing (the 0x3A use gate [M 0x1a5170:158-166]),
Lock_Activate 0x1aed00's kind-0 arm (Drones.lock_use), Lock_Update case 3/4's Keypad tick +
case 1's channel hold (already looped), C_KEYPAD_Handler 0x216810's digit handler
(Drones::keypad_digit), KeyCodes (Locks_Init 0x1b0548 over a local stream; the walk test
overrides with the snapshot's own table [L ee_start_0700000a.bin @0x2beff8 — code index 1 =
"3078"]). The mission walk un-ignored and GREEN end-to-end on real input: the keypad E, the
game's own digits, ch 0x30, the door's rising-edge unlock + auto-activate [M 0x1363c8:101-107],
the doorway crossing (z −18.7, all rays clear at state 4 [D probe]), the ring legs, the hatch E
→ 0700000b (15247 ticks, stuck 0, falls 0). walkreal_0700000a_live 7/0/2; nf-game 947/0/9;
interactables 7/0/0; keyboard 1/0/0. The FULL suite's chunked sweep did not fit the window
(the sandbox kills background jobs — the -3 report's finding); the next message runs it.

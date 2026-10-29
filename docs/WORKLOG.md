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

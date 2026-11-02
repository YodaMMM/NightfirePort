# level-14-15-real-health-finish — two real-health walks with route-side blockers

Branch from origin/master.

1. walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage is ignored: the route's live-wire check asserts Bond's health is 100 ("no live-wire hurt with +0x56 configured [M 0x1adfe8:99-101]") but on real health it reads 33 from earlier fights. Make the check measure the wire's own hurt (health before/after the crossing), not the absolute value, and finish the walk.
2. walkreal_07000015_live::walk_spawn_to_the_omega_sector_exit_real_damage is ignored: a -27.73 world hit at frame 5067 (a laser-mine fence blast — the walk crosses a live beam), then drone #5 at 4.2 u kills Bond at (0.0, 41.3, 23.9). Route the walk the way a player does: shoot the fence mines from range first (a_p2k_round_from_10_units_explodes_the_door38_fence shows how), then take the guard.

Real input only, no health writes. Un-ignore both; run walkreal_07000014_live, walkreal_07000015_live, every real-damage walk, then the full suite.

---
claimed-by: sandbox1 at 2026-11-02T17:02Z

---
## Report (done, 2026-11-02T19:35Z)


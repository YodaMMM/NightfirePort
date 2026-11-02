# level-0d-yard-climb-cover — the 0d real-health walk's climb under the sniper

Continue from origin/master (it now carries level-0d-yard-sniper-los: no LOS bug in the port; the ground leg is fixed; probe_0d_los_grid maps what drone #31 sees).

walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage stays ignored: the climb leg is exposed to drone #31 (placed sniper, weapon 0x1e, 16 per hit on Easy, 180-tick cadence).

Do:
1. Use the LOS grid to pick the climb timing and approach a player would: climb between the sniper's shots (the cadence is fixed), from the side the grid shows covered, or take the sniper out first from cover if the level allows a line to him.
2. Real input only, no health writes. Un-ignore the walk; run walkreal_0700000d_live, every walkreal_*_live, then the full suite.

Rules: shared test helper changes (crates/nf-view/tests/common/) must be opt-in per level (and the code must check the flag); re-run every walkreal_*_live before finishing.

---
claimed-by: sandbox1 at 2026-11-02T22:33Z

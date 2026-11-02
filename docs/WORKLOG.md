## level-0d-yard-climb-cover (2026-11-02)

Branch s/level-0d-yard-climb-cover from origin/master (0ea03862, carries level-0d-yard-sniper-los).
Goal: un-ignore walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage. The ground leg is
fixed (sniper-los), but the climb leg is still exposed to drone #31 (placed sniper, weapon 0x1e,
16 per hit on Easy, 180-tick cadence) — the real-damage twin takes ~5 snipers (~100 dmg) on the
climb/wire. Plan: use probe_0d_los_grid to pick the climb timing and approach a player would —
climb between the sniper's shots (fixed cadence), from the covered side of the grid, or take the
sniper out first from cover if the level allows a line to him. Real input only, no health writes;
then un-ignore the walk, run walkreal_0700000d_live, every walkreal_*_live, and the full suite.

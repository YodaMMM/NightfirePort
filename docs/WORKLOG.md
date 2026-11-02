## level-0d-yard-climb-cover (2026-11-02)

Branch s/level-0d-yard-climb-cover from origin/master (0ea03862, carries level-0d-yard-sniper-los).
Goal: un-ignore walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage. The ground leg is
fixed (sniper-los), but the climb leg is still exposed to drone #31 (placed sniper, weapon 0x1e,
16 per hit on Easy, 180-tick cadence) — the real-damage twin takes ~5 snipers (~100 dmg) on the
climb/wire. Plan: use probe_0d_los_grid to pick the climb timing and approach a player would —
climb between the sniper's shots (fixed cadence), from the covered side of the grid, or take the
sniper out first from cover if the level allows a line to him. Real input only, no health writes;
then un-ignore the walk, run walkreal_0700000d_live, every walkreal_*_live, and the full suite.
State at the deadline: the walk stays #[ignore]d. Delivered: probe_0d_climb_cover (the shooter
map + the reverse-ray matrix + the enablement timeline) and the strip-mouth kills in the walk —
the perch #31 kill from the drop point is FREE (19 ticks, one tap, pool 100->100; the non-damage
twin passes the full mission with the kills in). Found: the walker's 0.02 rad aim gate is wider
than a guard's 0.3 u torso box beyond ~15 u (new local shoot_far, range-scaled gates); the
aim-mode fire never lands steep shots (hip-fire does); the armour cannot absorb bullets (pain
types 1..7 armourFraction 0 [M 0x1902c8]); the west roof wakes at the platform drop and has NO
clear ground ray (the canopy); #33's clear window closes by the drop (the dock face eats the
steep shots east of x ~192). Next levers: kill #33 from the PLATFORM before the drop (the flat
shot), then climb immediately (the roof's ~2 hits are the honest cost), un-ignore, run all walks.

## level-0d-yard-climb-cover-2 (2026-11-02)

Branch s/level-0d-yard-climb-cover-2 from origin/master (0ea03862) + merge of
origin/s/level-0d-yard-climb-cover (its code was not on master yet: the perch kill, shoot_far,
probe_0d_climb_cover). Goal: apply the report's levers — kill #33 from the PLATFORM before the
drop (the flat shot), then climb immediately, eat the honest roof/wire cost, win the deck fight,
take the bay drop — and un-ignore
walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage on real input only.

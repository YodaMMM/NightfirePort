# level-16-squad-phase-real-damage — survive the pit-squad phase on real health

Branch from origin/master.

`walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission_on_real_damage` is ignored: the route, both shelters, the chaser's kill and pulse 2 all survive real damage, but the pit-squad phase (the 4-squad + pit shooter #8 at the room's door) still kills Bond at ~48 health.

Do:
1. Measure why: the squad's per-guard accuracy, fire rate and damage against the game's drone weapon code [M], and the cover the room offers. If the port's guards hit harder, more often or from places the game's can't, that's a port bug: fix it and cite the decompile.
2. If the port matches the game, give the walker a player's tactic (cover, timing, kill order) that the room really offers; no health writes, no teleports.
3. Un-ignore the test; it must pass on real damage. Run walkreal_07000016_live, fix_finale_live, then the full suite.

---
claimed-by: sandbox1 at 2026-11-02T05:22Z

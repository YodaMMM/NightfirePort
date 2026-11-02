# level-0d-yard-sniper-los — the 0d sniper sees the whole yard

Branch from origin/master.

walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage is ignored: drone #31 (placed sniper, weapon 0x1e) fires the game's law exactly, but the port gives it an unbroken line of sight to every cell of the yard (x 180-216, z -224..-268; probe_0d_los_grid on master), so Bond is hit the whole way and dies at tick 6549.

Do:
1. Check the yard's occluders: which world geometry and objects (containers, crates, cranes, walls) should block the drone's LOS in the game [D placements/collision], and whether the port's drone LOS ray tests them [M the drone visibility/LOS call and its collide mask]. A missing occluder class or wrong mask is a port bug — fix and cite it.
2. If the LOS really is open in the game, route the walk through the cover the yard offers (real input, no health writes).
3. Un-ignore the walk; run walkreal_0700000d_live, the drone LOS/sight tests, every real-damage walk, then the full suite.

Rules for this task:
- If you change anything under crates/nf-view/tests/common/ (walker, damage_log, walker_b2, ...), the change must be opt-in for your level (a flag your test sets) and you must run EVERY walkreal_*_live binary before finishing — shared-helper edits broke other levels three times on 11-02.
- A player-visible fix ships with a real-input test that reaches the spot from the level start (no teleport after load), so the passing test is the proof.

---
claimed-by: sandbox1 at 2026-11-02T20:01Z

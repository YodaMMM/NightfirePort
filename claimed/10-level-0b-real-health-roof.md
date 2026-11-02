# level-0b-real-health-roof — finish 0b on real health after the corridor fix

Branch from origin/master (it carries the GT wall-gun fix).

walkreal_0700000b_live::walk_fresh_spawn_to_the_mission_end_real_damage is ignored: the corridor crossing now costs 44.8 (100 -> 55.2, the game's wall-gun law), and the run dies at tick 16652 on the roof — the open-floor fights and walker stalls bleed the rest.

Do:
1. Read the damage log for the roof death (who, weapon, range, how often) and check those guards' fire against the game [M].
2. Use the level's own health/armour pickups on the route and a player's tactics (cover, kill order, no stalls in the open). Fix the port only where it differs from the game, with citations.
3. Un-ignore the walk; it must reach the mission end. Run walkreal_0700000b_live, every real-damage walk, then the full suite.

Rules for this task:
- If you change anything under crates/nf-view/tests/common/ (walker, damage_log, walker_b2, ...), the change must be opt-in for your level (a flag your test sets) and you must run EVERY walkreal_*_live binary before finishing — shared-helper edits broke other levels three times on 11-02.
- A player-visible fix ships with a real-input test that reaches the spot from the level start (no teleport after load), so the passing test is the proof.

---
claimed-by: sandbox3 at 2026-11-02T19:14Z

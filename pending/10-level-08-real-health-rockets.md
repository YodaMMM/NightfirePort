# level-08-real-health-rockets — finish 08 on real health without self-damage

Branch from origin/master.

walkreal_07000008_live::walk_the_whole_mission_on_real_input_real_damage is ignored: Bond dies at tick 26696 at (111.0, 13.5, -179.7). level-08-long-range-hits proved the port's fire law matches the game and the fatal hits are Bond's OWN rockets (splash at close range during the bottom-station fight).

Do:
1. Make the 08 walk's fights pick a weapon and range the way a player would: no rocket at targets inside the blast radius [M the rocket's blast radius/damage]; switch to the gun for close guards.
2. Un-ignore the walk; it must reach the mission end on real health. Run walkreal_07000008_live and every real-damage walk, then the full suite.

Rules for this task:
- If you change anything under crates/nf-view/tests/common/ (walker, damage_log, walker_b2, ...), the change must be opt-in for your level (a flag your test sets) and you must run EVERY walkreal_*_live binary before finishing — shared-helper edits broke other levels three times on 11-02.
- A player-visible fix ships with a real-input test that reaches the spot from the level start (no teleport after load), so the passing test is the proof.

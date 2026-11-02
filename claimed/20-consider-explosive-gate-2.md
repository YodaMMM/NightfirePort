# consider-explosive-gate-2 — the grenade flee gate without the 11 corridor regression

Branch from origin/master; merge origin/s/consider-explosive-gate in first.

That branch ports the game's flee-state gate [M 0x1431c0:40-67] (guards in aim/fire states now flee a live grenade), and nf-game is green, but two tests go red on master with it:
start_corridor_guard_07000011_live::idle_bond_is_approached_and_shot_at_every_selectable_difficulty and ::in_position_the_guards_step_aim_as_well_as_fire. The corridor guards keep fleeing grenades the port lets them lob — and that lob is marked [S] (suggested, not proven) in the port.

Do:
1. Prove when the game's drones throw grenades at all [M the drone grenade-throw decision: weapon/prop gates, range, LOS, cooldown, difficulty], and whether the 07000011 start-corridor guards ever throw one in the game. If the port's lob is not the game's, fix or remove it with citations.
2. With the faithful throw and the faithful flee gate together, both corridor tests must pass without changing their [L] pins (they were taken from the game).
3. Run nf-game, start_corridor_guard_07000011_live, every grenade/consider test, every real-damage walk, then the full suite.

---
claimed-by: sandbox6 at 2026-11-02T15:47Z

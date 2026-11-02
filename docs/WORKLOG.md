
## consider-explosive-gate-2 (started 2026-11-02 17:11 UTC)

Branch cut from origin/master 524924db (origin/s/consider-explosive-gate f4d40f39 is already an
ancestor — the flee gate [M 0x1431c0:40-67] is in). Task: the two red corridor-11 tests
(idle_bond_is_approached_and_shot_at_every_selectable_difficulty, in_position_the_guards_step_aim_
as_well_as_fire) fail with the faithful flee gate because the port's corridor guards lob grenades
— a lob marked [S] in the port. Plan: (1) pin the game's grenade-throw decision [M: the
CanThrowGrenade/ConsiderExplosive gates, weapon/prop/range/LOS/cooldown/difficulty] and whether
07000011's start-corridor guards can ever throw; (2) fix or remove the port's lob with citations;
(3) both corridor tests pass without touching their [L] pins; run nf-game, corridor-11, every
grenade/consider test, every real-damage walk, full suite.

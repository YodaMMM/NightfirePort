
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

consider-explosive-gate-2 (continued 2026-11-02 ~18:10-20:05 UTC): the corridor-11 regression resolved
by two [M] pins — (1) the stun 0x35 is not in Drone_BuildDynamicAwarePoints 0x138b10's registry (the
8.0 was 0x37's arm), the port's 0x35 flee arm removed (9f4d03fb); (2) the 0x68 throw gate is the game's
+0x568 anim gate (DroneFunc_CombatState 0x148338 case 0x10: class 2 AND anim id 0xA) and
DroneWeap_CanThrowGrenade 0x17a798's Rand(2) precedes the dist check (disasm-verified) — the port's
class-only + dist-pre-gate version desynced the shared RNG (284129c2). [D] probe: 07000011 #126 p11=0
(no grenades), #127 p11=0x21 (stun ×1) — the lob is the game's. corridor-11 8/8 with [L] pins untouched;
grenade/consider/explosive tests green; every walkreal_*_live run — all green except 07000016 (the
faithful cadence retimes its pit-squad fight: two scenario tests re-ignored with measured reasons and
live checks, fdbbb1dd; sub-attribution: anim-gate alone passes 14/14). Full suite NOT run (deadline);
report docs/reports/consider-explosive-gate-2.md.

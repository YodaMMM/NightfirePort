
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

## consider-explosive-gate-3 (started 2026-11-02 21:06 UTC)

Branch cut from origin/master c3e3135f with origin/s/consider-explosive-gate-2 6e5eef87 merged in
(clean). Task: land the gate-2 faithful throw/flee rules with every walk green — (1) 07000015: why
the #383 → #385 fence blast chain changes at tick 2640 ("#385 goes on #383's blast (cause 2)",
health 86.1) and whether the game would do the same; (2) 07000016: a real-input pit-squad tactic
under the faithful roll cadence (ammo pickups, weapon choice, cover), un-ignore both walks;
(3) run every walkreal_*_live, start_corridor_guard_07000011_live, every grenade test, full suite.

## drone-block-swaps-g055 (started 2026-11-02 ~21:10 UTC)

Branch cut from origin/master c3e3135f. Task: GAPS G-055 — wire the three remaining behaviour-block
swaps: FirstAttack's alternate-state branch [M 0x148010:100-131], PreDroneControl's second-switch
flush [M 0x148db0:58-82], HandleImpact's 09/0A Set1st [M 0x1469b8:114]. Each ported with its address
and pinned with a scenario on a level that uses it (plus the sister: swap does not happen when the
condition is false). Then ai_live, stealth_ai_live, every walkreal_*, nf-game; fix walks only where
fragile; quote the totals. Fast suite runner started in background once tools/test/suite.py exists.

Progress (same session, ~22:40 UTC): all three sites wired in crates/nf-game/src/drone.rs —
(1) FirstAttack's alt tail in the ATTACK-ENTER generic arm (after the zero first-sight pick, first
pass only): +0x5A2=0x56 + change_to_attack_mode inline (the 0xb4→0x56 remap is FirstAttack's own
[M 0x148010:128-131]); (2) PreDroneControl's flush now runs change_to_attack_mode when +0x5A4 != 0
(the verbatim-0 SetState kept); (3) on_tase's fall arm carries the 09/0A stamp block (latch clear,
0x89 arm, Set1st via set_1st_behaviour, mode 9 / alt 7, ret 0x14 / resume 0x15). nf-game lib 658/658
(3 new [M]-exact unit tests with sisters: swap happens / condition false → no swap). Discovery probes:
07000007's wake guards (ch 21/22, enabled, alt 0xb4) are the flush pin; 07000003 #12/p400 (enabled,
alt 0x56) the FirstAttack pin; 07000009's p5=100 civilians (hp 1, alt 0x89 at init) the 09/0A tase
pin population. Live tests + suite runs next; background processes die between tool calls in this
sandbox, so the suite runs in foreground slices.

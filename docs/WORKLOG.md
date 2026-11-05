# WORKLOG

## fx-scrape-and-impacts
Started 2026-11-05 00:52 UTC. Branch s/fx-scrape-and-impacts (cut from origin/master 17a784980).
Task: port rows 7, 8, 11 and 13 of the audit-effect-calls "Unported calls" table — rigid body
scrape dust (RB_Update 0x1d6e38), anim script commands 6 and 7 (AnimProcessScriptCmds
0x1c1b38), the laser env puff (Effect_Laser 0x1cef38:117) and the wall-puff velocity + the
HITDATA+0x51 roll (Effect_Bullet 0x1ce8a8). Merge origin/master first. Cite [M] for each call,
pin each with a test through the real path on real level data, report every value used. Shell
casings out of scope; do not change them.
Outcome (2026-11-05 02:12 UTC): rows 7, 11 and 13 ported with [M] disassembly citations; row 8
settled by the disc census as UNREACHED (no anim script on the disc carries ops 6/7 — ops
{0,1,4} only across every level archive + its 078/079/07A skin packs). Row 7: rb_update returns
the buoyancy puff position (Gas_Init 0.5/1.0, 0x2000129 kind 1, 0x40008080→0x80808008, life 16
[M disasm 0x1d6ee8]); pushed in drone.rs's kind-7 arm. Row 11: the laser env puff on
GameState&1 = the level code's bit 0 (0.05/0.1, SMOKE_QUAD kind 0, 0xc8c8c87f→0x20202006, life
21 [M disasm 0x1cf1ec], jitter = FRandHalf(0.1) ⊙ HITDATA+0x20 = the orthonormal-basis sum). Row
13: Vec_NormToRot 0x1e31b0 ported + Gas.rot (init_ex_rot) — wall puffs spawn oriented along the
normal, floor puffs with the negated euler by +0x1b; the row's +0x51/0x80 half was already in
penetration_roll_node (the module doc note was stale — updated the wall-puff comment). Tests:
nf-game lib 663/0 (the new wall-puff rotation test), anim_fx_cmd_census 1/0, rigid_body_wiring
1/0, laser_gas_mesh 4/0, blast_smoke 2/0, break_debris 4/0, grenade 1/0, impacts 6/0,
fix_weapon_effects 3/0, weapon_fx_levels 5/0, impact_remainder 2/0. Full suite not re-run (the
prior session's sandbox ran the linker out of disk; the changed-code binaries above all pass).

## level-0b-walk-after-scrape-draw
Started 2026-11-05 07:20 UTC. Branch s/level-0b-walk-after-scrape-draw (s/fx-scrape-and-impacts rebased onto origin/master a89818235).
Task: land s/fx-scrape-and-impacts — the laser env puff jitter and the wall-puff roll shifted the
shared RNG stream, so Bond dies at the end of walkreal_0700000b_live's real_damage_agent and
real_damage_00agent at (2.22,58.08,36.54), under fire from drone #4 at 4.5 units. Make the walks
survive the way a player would (fight or take cover there); do not remove or reorder a draw.
Impact tests must count bullet holes only (guards' footprints share the decal pool). Run
walkreal_0700000b_live, impacts_live, qclaw_live, impact_remainder_live.
Outcome (2026-11-05 07:55 UTC): s/fx-scrape-and-impacts rebased onto origin/master a89818235
(clean) and landed on s/level-0b-walk-after-scrape-draw. The walk deaths fixed the player's way:
the deck sprint now runs THROUGH the parachute take slot (the take raises ch 0x3F on the pass —
the touch law is a per-tick distance+LOS check [M Pickup_Update 0x18e148:202-220]), so the walked
take-slot stop's return-fire gate can no longer stand Bond in a blocked dart fight at the deck's
south-west while drone #4 (weapon 0x18) chews from 4.5 u — the real_damage_agent/00agent deaths
at tick 16047 (2.22,58.08,36.54) were that stand [L the NF_RF_TRACE fight#9 log: 180+ ticks,
blocked true, health 53.2 → 0]. No draw removed or reordered; the tranq fights stay on the legs
where their target is reachable. The impact tests now count BULLET HOLES only — the wait keys on
an EffectInfo+4-mesh decal [M Effect_Bullet 0x1ce9c8] and the sky sisters assert no +4-mesh
decal, since the guards' footprints share the 0x3496e0 pool with the +8/+0xc boot meshes [M
Effect_Player 0x1cec48:84-107]. Green: walkreal_0700000b_live 7/0 (real_damage 62.8, agent 42.0,
00agent 27.5 left; mission complete tick 16076, stuck 0), impacts_live 5/0, impact_remainder_live
3/0, qclaw_live 3/0, nf-game lib 667/0 (5 ignored), laser_gas_mesh 4/0, blast_smoke 2/0,
break_debris 4/0, grenade 1/0, rigid_body_wiring 6/0, npc_doors 6/0 (1 ignored),
fix_guard_misc 2/0, walkreal_07000001 8/0, walkreal_0700000d 1/0 (2 ignored); nf-formats 136 ok
+ the 3 allowed disc_golden fails; nf-audio 28/0. The nf-view suite whole (959 test binaries)
did not fit the window; no src/ changed on this branch beyond the scrape branch's, whose
changed-code binaries all ran above.

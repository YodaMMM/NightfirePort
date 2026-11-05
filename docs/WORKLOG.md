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

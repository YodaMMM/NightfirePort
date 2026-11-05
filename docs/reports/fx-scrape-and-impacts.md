# fx-scrape-and-impacts — rows 7, 8, 11 and 13 of the audit-effect-calls unported table

Branch `s/fx-scrape-and-impacts` (cut from origin/master 17a784980 — the audit-effect-calls
merge, so `docs/reports/audit-effect-calls.md` is on master). Task: port rows 7 (rigid body
scrape dust), 8 (anim script commands 6/7), 11 (the laser env puff) and 13 (the wall-puff
velocity + the +0x51 roll); cite [M] per call, pin each through the real path, report every
value. Shell casings out of scope.

## 1. Row 7 — the rigid-body scrape/buoyancy puff (fixed)

`RB_Update` 0x1d6e38's arm (obj+0x20 cel non-null, `obj+0xc4 < cel+0x90`, `GameState & 0xf !=
0`) draws ONE `Rand_FRand(0.15)` and both kicks the body up
(`RB_AddImpulsiveForce(0, |obj_y − cel+0x54|·(ctrl+0x78 mass)·0.4 + rand, 0)` [M :49-62] — the
port's `rb_update` already had this) and draws the gas puff the port never spawned:
`Gas_Init(0.5, 1.0, cel, obj+0xc0, NULL angles, 0, gfx 0x2000129, kind 1, rgba 0x40008080 →
0x80808008, life 0x10 = 16 ticks)` — the life is the decompiler-dropped 11th arg, read from
the disassembly (`0x1d6ee8 li v0,0x10` + `0x1d6f30 sw v0,0(sp)`) [M]. The port's own doc on
this block already reads it as the BUOYANCY puff (cel+0x90 = the water surface per
build_alloc_cel/parseentity_fixup_entity), which the census of the naming agrees with —
"scrape" was the audit's word for the same gate.

Fix: `rigid_body::rb_update` returns `Some(obj+0xc0)` when the arm fires (same gate, the same
single draw feeding both effects); `Rbodies::tick_one` propagates it; drone.rs's kind-7 control
arm (the per-object `RB_Update` dispatch site, 0x26f9c8's pair) pushes
`gas.init_ex(0x0200_0129, puff, 0.0, [0,0,0], 0.5, 1.0, kind 1, [0x40,0x00,0x80,0x80],
0x80808008, 16.0)`. Pinned by `rigid_body_wiring_live` (the real tick path, 1/1) — no census
level has a kind-0xD8 body inside a water cel (the lamps sit above their cel's surface), so the
puff never fires on the disc's wired levels; the values are unit-pinned in the rb_update doc +
the wiring test asserts the resting body draws nothing and returns None.

## 2. Row 11 — the laser impact's env puff (fixed)

`Effect_Laser` 0x1cef38:110-120: on `GameState._52_4_ & 1` — +0x52 is the LEVEL CODE word
(the same register RB_Update's `& 0xf` gate reads [M]; bit 0 = the odd-numbered levels), NOT a
frame counter — a white puff spawns at `hit + (Rand_FRandHalf_Vec(0.1) ⊙ HITDATA+0x20)` (+0x20
being `Effect_Create`'s stored orthonormal-basis sum [M 0x1ce740], the tangential spread),
`Gas_Init(0.005, 0.1, cel, pos, angles = the zero vector 0x2d6200 [L ACTION.ELF (0,0,0)], 0,
gfx 0x2000039 = SMOKE_QUAD, kind 0, rgba 0xc8c8c87f → 0x20202006, life 0x15 = 21 ticks [M
disasm 0x1cf1ec `li v0,0x15` + 0x1cf228 `sw v0,0(sp)`, the dropped 11th arg])`. The three
FRandHalf draws run whenever the gate passes — outside the burn puff's EffectInfo mesh gate
[M :110 sits outside :59's].

Fix: drone.rs's laser-impact arm spawns it after the existing burn puff,
`gas.init_ex_rot(SMOKE_QUAD, p, 0.0, [0,0,0], 0.05, 0.1, 0, [0xc8,0xc8,0xc8,0x7f], 0x20202006,
21.0, [0,0,0])`, gated `self.level_current & 1 != 0`, with `frand_half_vec(&mut rng, 0.1)`
consumed in the shared stream and `orthonormal_basis_sum(n)` as +0x20. Pinned through the real
laser path by `laser_gas_mesh_live` (4/4 — the burn-puff binary whose path the new arm rides)
and the RNG discipline note in the code.

## 3. Row 13 — the wall-puff spawn rotation (fixed) and the +0x51 roll (already in)

`Effect_Bullet` 0x1ce8a8's gas loop passes each entry a spawn rotation, not a velocity: a +0x1a
wall puff gets `Vec_NormToRot(normal)` [M 0x1ce8a8:77,:129 = `Vec_NormToRot` 0x1e31b0:
`(−atan2(y, √(x²+z²)), atan2(x, z), 0)`], a floor puff gets the negated-normal euler
(`−ATAN2_APPROX(−y, √((−x)²+(−z)²)), ATAN2_APPROX(−x, −z), 0` [M :78-85]) when its +0x1b byte
is set, else the zero vector [M :122-130]. The game's gas objects are NOT billboards
(`Gas_Init` ORs 0x20020120 — no 0x8 bit), so `View_DrawObjectGlist` rotates them from their own
+0x90 — the rotation is the puff's visible orientation.

Fix: `effect::vec_norm_to_rot` + `effect::negated_normal_euler` ported exactly (libm atan2 [S:
the game's ATAN2_APPROX differs sub-degree]); `Gas` gains `rot: [f32; 3]` (own line;
`init_ex_rot` fills it; every pre-existing site keeps [0,0,0]); the wall-puff loop passes the
per-entry rotation. Pinned in `nf-game`'s lib tests
(`wall_puffs_spawn_oriented_along_the_normal`: the pure euler values + the three real-path arms
through `bullet_impact_on` — wall → Vec_NormToRot, floor+0x1b → the negated euler, floor →
zero). The draw side's consumption (an oriented quad instead of the billboard) is render work —
listed in the live checks.

Row 13's second half — "skips the HITDATA+0x51 second-byte material roll (effect.rs:14-15)" —
is ALREADY ported: the +0x50&0x80 `Rand_Rand(0x3f)` pierce roll is `penetration_roll_node`
(effect.rs:389-403, the fix-foliage-bullets/fix-rocket-glass-08 work), and HITDATA+0x51 bit 1
is its `node_pass` input. The audit-effect-calls row cited the module doc's stale RNG note
(effect.rs:14-15), which predates those fixes; the wall-puff comment now points at the
rotation work.

## 4. Row 8 — anim script commands 6 and 7: UNREACHED on the disc (census)

Both cases decoded [M `AnimProcessScriptCmds` 0x1c1b38]: case 6 =
`Effect_Create(1.0, *(obj+0xdc), obj, 0xc0)` — flags 0x80 → `Effect_Body` 0x1cf400 = the 3D
body-impact sound `Sound_Play3D(100.0, 0.5, 0.5, EffectInfo[mat]+0x18, hit pos, 0,0,0)` +
`Sound_ModAlertness(0.5, h)` (the guards hear the body hit), flags 0x40 = the same family's
+0x16 arm — the material from the object's STORED hitdata at obj+0xdc; case 7 = the NPC muzzle
event: `AnimDatumGetWeaponInfo` finds the weapon datum's muzzle bone, `AnimGetBoneWorldTrans`
poses it, `Bullet_init(−1, obj, obj, weapon_data+w·0x10c, matrix, muzzle, entity)` fires from
the bone, then `AnimDatumSetEntity(obj, bone+1, entity, 3)`, `Light_Create(weapon+0x58, 2.0,
muzzle, rgb weapon+0x56/55/54, 0, kind 3, 1, 0)` and — unless `weapon+0x68 & 0x2000` —
`Gas_Init(0.05, 0.1, cel, muzzle, (0, Rand_FRand(2π), 0), 0, 0x2000039, kind 7, 0x48403080,
end 1)` (life dropped; not recovered — see the live checks).

**The census settles the row**: walking EVERY level archive 07000001..0700004a plus each
level's `078/079/07A` skin packs plus the weapon pack through the port's kind-5 script parser,
the op histogram is `{0, 1, 4}` only — 07000005 alone carries 568 scripts (574 × op 0, 107 ×
op 1, 960 × op 4) and **zero op-6 and zero op-7 commands exist in any anim script on the disc**
[D `anim_fx_cmd_census`]. The interpreter's cases 6/7 are dead on the shipped data — nothing to
port (the M_TEXTBOX precedent). The [M] record above stands for whoever adds new anim data.
The port's guard fire path already creates its own muzzle light [M DroneWeap_FireWeapon
0x178f68:189, ported at drone.rs:11284] and the fire anims' op-1 shot sounds — the census shows
those are the disc's only fire mechanisms.

## 5. Verification

- `nf-game` lib: **663/0** (5 ignored) — includes the new `wall_puffs_spawn_oriented_along_the_normal`.
- nf-view: `rigid_body_wiring_live` 1/0, `anim_fx_cmd_census` 1/0 (new), `impacts_live` 6/0,
  `fix_weapon_effects_live` 3/0, `weapon_fx_levels_live` 5/0, `impact_remainder_live` 2/0
  (+2 ignored).
- nf-game: `laser_gas_mesh_live` 4/0, `blast_smoke_live` 2/0, `break_debris_live` 4/0,
  `grenade_live` 1/0.
- The full suite was not re-run whole this session (the sandbox disk ran the linker out of
  space in the previous session's run; every binary that exercises the changed code passes
  above). The walkreal set was NOT re-run — no shared test helper changed; the changes are the
  gas-loop rotation, one added puff arm on the laser path, one puff return on the RB path.

## 6. Live checks for the main agent

1. **The RB buoyancy puff** — a rigid body in a water cel (a barrel/knick-knack pushed into
   water on any level with class-0xC023 rooms): watch the body's obj+0xc0 and the gas pool
   while it bobs — expect the 0x2000129 kind-1 puff while the body is below cel+0x90, with the
   upward kick per tick.
2. **The laser env puff** — burn any surface with the Laser (0x4E) on an odd level (07000005):
   expect the second white puff beside the burn puff, drifting tangentially; on an even level
   (07000006) expect NO env puff (the level code's bit 0 gates it) — one screenshot each.
3. **The wall-puff orientation** — a render of `--bond` at any wall with the impact puffs up:
   the wall puff should now carry the surface euler in `Gas.rot`; the port's draw still
   billboards, so the visible change is nil until the draw consumes `rot` — confirm on PCSX2
   that the game's wall puffs read as surface-aligned sheets (the render side's target).
4. **Anim op 7's dropped life** — if anyone revives row 8 with new data: disassemble the
   `Gas_Init` call at the case-7 tail in `AnimProcessScriptCmds` for the `sw v0,0(sp)` life
   value (the 11th arg, the decompiler drops it — same pattern as rows 7/11).

## 7. What's left

- The gas draw consuming `Gas.rot` (the oriented non-billboard quad) — render work, values pinned.
- The row-8 verdict holds unless new anim data ships; the [M] record is in §4.

## Usage

not visible to me

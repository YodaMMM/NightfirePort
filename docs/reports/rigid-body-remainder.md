# rigid-body-remainder — report

Task: port the rigid-body functions the port never names (RB_CalcTorque 0x1d7658, RB_ApplyWind
0x1d7c78, RB_AddImpulsiveForce 0x1d7d40, RB_AddImpulsiveForceAtPoint 0x1d7d58, RB_ToWorld
0x1d7e18, RB_ToBody 0x1d7eb8, RB_apply_omega 0x1d7f50, RB_VecSpecial 0x1d8038), find every caller
and the levels that place kind-0xD8 bodies, port the update + the force calls that reach it, with
rest/moved tests.

## What was done

Prior art on master: `crates/nf-game/src/rigid_body.rs` already held the four init functions
(RB_Init/CuboidMomentsOfInertia/Build/SetState — ai-spawned-objects + review). This session adds
the **evolution core**, each function from its decompile and, where the C was VU-opaque, from its
disassembly (repo `tools/crosscheck/mipsdis.py`):

- `RbState` — the runtime slots: +0x00 position, +0x10 omega, +0x20 velocity, +0x30 angular
  acceleration, +0x40/+0x50 the impulse accumulators, and the obj+0x90 matrix (+0x30 translation)
  the draw reads [M RB_Update 0x1d6e38 writes rb+0x130 from obj+0x20 and EulerEvolve writes
  *(rb+0x74)+0x30 = the integrated position — the moving body moves on screen through this row;
  the render check itself is the main agent's].
- `rb_add_impulsive_force` 0x1d7d40 (+0x40 += f), `rb_add_impulsive_force_at_point` 0x1d7d58
  (+0x40 += f; +0x50 += (point − translation)×f) [M both decompiles complete].
- `rb_to_world` 0x1d7e18 (the column apply) / `rb_to_body` 0x1d7eb8 (the row apply) [M].
- `rb_vec_special` 0x1d8038 [M disasm: out = f·(a − b·c)].
- `rb_apply_omega` 0x1d7f50 [M: half = mag·dt·0.5; delta = (cos(half), omega·sin(half)/mag);
  quat = delta ⊗ quat]. The game's quaternion layout is (w,x,y,z) — +0x64 holds the identity's
  1.0 [M RB_Init 0x1d7ad8] — my first cut had it w-last and the test caught it.
- `rb_apply_impulses` 0x1d6ff8 [M disasm]: torque arm = ToBody → x·(+0x8c)=1/I0, y·(+0x90)=1/I2,
  z·(+0x94)=1/I1 (the game's cross-wired store order), clamp ±4, ToWorld, omega +=; linear arm
  (type 0 only): pos += (+0x7c 1/mass)·(+0x40).
- `rb_euler_evolve` 0x1d7810 [M disasm]: flag==0 applies impulses then zeroes both accumulators;
  angular damping = omega·expf(−dt/+0xa4) (the game's sum·⅓ arithmetic cancels); type 0:
  velocity = `RB_VecSpecial(pos, *(+0x60), (expf(−dt/+0xa0)−1)/dt, +0xa0)` with the reference
  vector = WldGravity (the +0x60 table for the box type is 0x2d8890 = WldGravity itself
  [M RB_Create:74 + symtab 002d8890]); type 0 only: pos += dt·vel and the drawn translation
  follows. WldGravity = 0xc11ccccd = −9.80078 [M bootup 0x1c7e78:73, Player_Init 0x1910f8:153;
  the port's `grenade::WLD_GRAVITY` (0, −9.8, 0) matches within f32 rounding].
- `rb_apply_wind` 0x1d7c78 [M]: force = `Env_GetWindVel` output · (+0x7c)·REC_FRAME_RATE; type 0
  accumulates it, other types add it at the adjusted point (obj+0xc0 x, obj+0xc4 + +0x9c − +0x13c
  y, obj+0xc8 z).
- `rb_update` 0x1d6e38's sim part: the heat puff — cel set, obj+0xc4 below cel+0x90, and
  `GameState & 0xf != 0` (the level code's low nibble, true on every census level [M GameState ==
  the level code per ssys.rs]) — force (0, |obj+0xc4 − cel+0x54|·mass·0.4 + Rand_FRand(0.15), 0),
  the shared-RNG draw ONLY when the gate passes [M 0x1d6e38:0x1d6e88-0x1d6eb0]; then wind, then
  the evolve. The Gas_Init smoke is the effect layer (the port's effect module, not the sim).
- `rb_explosion_hit_arm` — the class-5 (explosion) hit arm of `RB_RestingContact 0x1d7168`
  [M 0x1d7168:0x1d7238-0x1d72e4]: impulse = hit_vel·(hit+0x8·0.5) into +0x40, (hit+0x40 −
  translation)×impulse into +0x50, after the game's Sound_Play3D (audio layer, not sim).

## Callers and reachability ([M] except as noted)

- `RB_Create` 0x1d6bf0 ← `parsemap_create_dynamic_objects` 0x1d04f0 case 0xD8 (level load).
- `RB_Update` 0x1d6e38 and `RB_CollisionHandler` 0x1d7a80 sit adjacent in the per-kind dispatch
  table at 0x26f9c8/0x26f9cc {update, collision} [M ELF scan; no direct caller — pointer
  dispatch]. The table holds the other kinds' updates (Door_Update 0x1363c8 at 0x26f9e8,
  SP_Update 0x1dbda8 at 0x26f9d8), so the port's wiring point is its per-kind update + hit
  dispatch (the remainder, below).
- `RB_EulerEvolve` ← RB_Update; `RB_ApplyImpulses` ← EulerEvolve; `RB_ApplyWind` ← RB_Update;
  `RB_AddImpulsiveForce` ← RB_Update/RB_ApplyWind/AtPoint; `RB_ToWorld/ToBody` ← ApplyImpulses;
  `RB_apply_omega`/`RB_VecSpecial` ← EulerEvolve; `RB_CalcTorque` ← RB_RestingContact
  ← RB_CollisionHandler. Nothing in the RB family is called by the explosion or bullet code
  directly — blasts and shots reach bodies through `RB_CollisionHandler`'s hit list (obj+0xd0),
  whose class-5 arm is the impulse ported above.
- Levels placing 0xD8 [D docs/systems/levels.md §RB rows]: **07000001, 07000002, 07000003,
  07000004, 07000006, 0700000d, 07000041** (the barrel placements on 06/0d carry type halfword
  0xFFFF [D, prior task's census]).

## Tests (crates/nf-game/src/rigid_body.rs, all [M]-cited)

11 passed (4 pre-existing init + 7 new): the census barrel (type 0xFFFF, mass 3) at rest is
**bit-identical over 600 `rb_update` ticks** (WldGravity never touches type != 0, position
integrates only for type 0, zero omega skips apply_omega — and the no-RNG-draw closure proves
resting bodies never touch the shared RNG); a blast at an off-centre point tips it along the
game's integration (impulse scale, r×f, the 1/I store order, the ±4 clamp, the expf damping, the
quaternion step, the drawn matrix leaving identity) with pos/trans unchanged (type != 0 has no
linear arm); the type-0 impulse arm (pos += 1/mass·impulse) and the VecSpecial damping formula
exact to 1e-6; the wind type-arm + adjusted point; the heat-puff gate (force exact; the RNG
closure panics on every closed-gate path); ToWorld/ToBody round-trip; the VecSpecial formula.

## What's left (the next session's work, in order)

1. **The dispatch wiring**: create the kind-0xD8 props as sim objects (the port draws them as
   static instances today) and tick `rb_update` per kind-table entry {0x1d6e38, 0x1d7a80}.
2. **RB_RestingContact 0x1d7168 / RB_RestingContactRot 0x1d73a0 / RB_CalcTorque 0x1d7658** — the
   contact response needs the port's collision sweep to produce the hit list (obj+0xd0) the way
   `Collide_CylinderIntersect` 0x1ecc00 does (the +0xB0 cylinder HITTEST, reached on 06/0d [D]).
   CalcTorque's basis/threshold structure is readable [M 0x1d7658] but unported.
3. **The EulerEvolve type != 0 up-righting tail** (0x1d79f4+, the Mat_GetUp omega correction
   with FRAME_RATE_MUL) — only reachable on tipped bodies, i.e. after (2).
4. **Env_GetWindVel 0x17c048** — the per-cel wind query; whether any census level's cels carry
   non-zero wind is an open [D] census.
5. Suite: this session ran the nf-game unit tests for the touched crate; the full four-crate
   suite did not fit the window (setup re-run + research consumed it) — run it on this tree.

## Live checks for the main agent

- `Quat_QuaternionMultiply` order: with a body already rotated (quat ≠ identity), apply a world
  omega and read *(obj+0x90): the port pre-multiplies (delta ⊗ quat). A gdbwatch on the matrix
  after a second-axis knock settles the order.
- The heat puff in game: place a body below a cel with +0x90 above it on a census level and watch
  rb+0x40 accumulate (0, force, 0) with the shared RNG advancing only while the gate holds.
- Any census level cel with non-zero wind (Env_GetWindVel output ≠ 0) — watch rb+0x40/+0x50.

## Usage

not visible to me — this session's environment shows no token, credit, cost or context-window
figures anywhere I can read.

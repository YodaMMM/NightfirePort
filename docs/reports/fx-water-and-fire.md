# fx-water-and-fire — the water ripple's callers + the flicker spark puffs

Branch `s/fx-water-and-fire` (cut from origin/master 17a784980). Rows 3 and 4 of
audit-effect-calls' "Unported calls" table. Shell casings untouched.

Note for the driver: master's merge 17a784980 dropped `docs/reports/` from the tree —
`docs/reports/audit-effect-calls.md` exists in commit 62756abf6 (an ancestor) but not in
master's checkout. I read the report from history; this file re-creates the directory.

## What's done

**Row 3 — Env_WaterRipple 0x17c118 wired into both of the game's production arms.**
The game's two callers are the flesh handlers' water arms, and both are **footstep-gated**,
not bullet-gated: the ripple runs only on a frame whose AnimObject event word (+0x60) carries
bit 0x4 (the stride accumulator's footstep, `AnimObjectUpdate` 0x1befd8:175-186) AND whose hit
material is the water pair:

- `Player_DealWithObjHit` 0x194458:65-85 [M]: a hit with `(mat & 0x3f)` in 0xd..0xe stores the
  0x60-byte HITDATA over the anim object (obj+0xdc, `amemcpy` :77) and, when +0x60 bit 0x4 is
  set (:80), calls `Env_WaterRipple(param_3)` (:81) before the hit-class dispatch.
- `NDrone2_DealWithObjHit` 0x13fff0:72-84 [M]: the same gate reads the drone's anim object
  (`Drone+0xc -> obj -> +0xdc -> +0x60 & 4`) and calls `Env_WaterRipple(param_3)` (:83).

Port wiring (the port has no monolithic DealWithObjHit; the arms live in their subsystems):

- guards: the script-footstep block, `crates/nf-game/src/drone.rs` (the `dr.footstep` event —
  the port's bit-0x4 equivalent — already probes the floor with `segment_surface` and plays the
  surface sound); the ripple call sits right after the sound, feeding the same probe's material
  and hit point (`feet+0.5 - 1.5·t`, the block's ground-contact y).
- player: the footstep block in `crates/nf-view/src/scene.rs:3578-3589` — the same block the
  footprint-decal arm (audit-effect-calls) extended — adds
  `nf_game::env::env_water_ripple(&mut self.drones.gas, m, ground_point)`.

`env_water_ripple` itself (`crates/nf-game/src/env.rs:230`) was already disasm-exact and
unchanged: gate `(material & 0x3f) - 0xd < 2` [M 0x17c120-0x17c138], then
`Gas_Init(0.5, 1.01, cel, hit+0x40, 0, 0, 0x0200_0129, kind 1, rgba 0x7f7f7f7f, end 2, life 63)`
— the life the decompiler drops is `sw $v0(0x3f), ($sp)` at 0x17c180 [M disasm; env.rs:226].

Nuance [S], documented at both call sites: the port's footstep probe is the grounding variant
(`no_water = true`), which drops only material-0xE triangles (world.rs `segment_surface` — "the
water planes never ground the player"). So 0xD floors ripple through the port's probes; a 0xE
surface hit (which the game's collide-pass hit can also be) never reaches the port's footstep
probe. The game gate is ported exactly; whether the game's footstep hit can be 0xE is a live
check below.

**Row 4 — Flicker_Update 0x1ab960's smoke puffs ported behind their exact (dead) gate.**
The spark branch [M 0x1ab960:117-129] requires ALL of: the map light found AND off
(`light+0x48 == 0`, :82), the placement's param-0 gate non-zero (`data+4`, piVar10[1]), and an
even frame (`GameState+0x34 & 1 == 0`; the decompile's `_52_4_` names DECIMAL 52 — the same
counter env.rs:246 cites; the sibling `_12_4_` level-id read at 0x13fff0:310 pins the base).
`Light_FindMapLight` 0x1cc3d0 is an 8-byte `return 0` stub on this build [M decomp
001cc3d0], so the branch can never run in the shipped game — the port now carries it with the
same structure (`find_map_light() -> Option<bool>` returning None with the [M] citation; the
tint's white arm + the channel stamp unchanged), so behaviour is byte-exact and a future
find-map-light port drops straight in.

The puff itself, pinned value-for-value by
`world_fx::tests::flicker_spark_matches_flicker_update_0x1ab960`:

- `Gas_Init(1.2, 0.8, cel, obj+0x80, 0, data+0x10 drift, FIRE_MESHES[Rand_Rand(5)], kind 4,
  rgba 0xffffffff, rgba_end 0x0a460003, life 85)` [M 0x1ab960:118-124]. The life the decompiler
  drops is `addiu $v0, $zero, 0x55` (0x1abc0c) stored `sw $v0, ($sp)` at 0x1abc34 before
  `jal Gas_Init` 0x1abc3c [M disasm, verified this session; same store pattern as the ripple's
  63 at 0x17c180].
- The mesh table at 0x2fcca0..0x2fccb0 = 0x020002d1, 0x020002d2, 0x020002d3, 0x020002d4,
  0x020002d5 [D — read from work/elf/ACTION.ELF byte-for-byte this session; matches
  docs/atlas/flicker.md].
- The drift (Gas_Init's velocity seed) is `Mat_GetUp` of `View_RotTransMatrix(orient, pos)` =
  matrix row 1 [M 0x1acb00:58-59; the rigid_body.rs:851 precedent] — `FlickerFx.drift`, built
  at attach from the port's existing `placement::view_rot_trans_matrix`.
- `Flicker_Create` writes data+4 = placement param 0 (the spark gate) [M 0x1acb00:56] —
  `FlickerFx.spark_gate`, read at attach.
- The game follows with `Particles_Create(5, obj+0x80, plane (0, -1, 0, 0))` [M 0x1abc44-68] —
  unported: the port has no particles system and `Particles_Update` 0x1cf528 self-kills through
  the stubbed `psiParticleUpdate` 0x111290 anyway [M; world_fx.rs module doc].
- RNG discipline: the game's `Rand_Rand(5)` mesh draw sits behind the dead gate, so the shared
  bit-exact stream gains nothing on this build; the unit test drives a local Rng.

Also fixed the doc slip at the tick header: `frame` = `GameState+0x34`, not "+0x52" (the
Ghidra `_52_4_` is decimal).

## How it was verified

- `cargo test --offline --release -j2 -p nf-game --lib -- --test-threads=1`: **663 passed /
  0 failed / 5 ignored** (includes the new `flicker_spark_matches_flicker_update_0x1ab960`,
  the existing `flicker_fades_white_and_the_signal_channel_stays_clear` (its FlickerFx literal
  now carries spark_gate/drift), and env.rs's `water_ripple_fires_on_the_water_materials`).
- `cargo test --offline --release -j2 -p nf-view --lib -- --test-threads=1`: **100 passed /
  0 failed** (the scene.rs footstep path compiles; every view lib test green).
- Values re-verified first-hand this session: Flicker_Update 0x1ab960 body, Flicker_Create
  0x1acb00 body, Env_WaterRipple 0x17c118 body, Light_FindMapLight 0x1cc3d0 stub, the
  Flicker_Update disasm around the Gas_Init call (life 85), the 0x2fcca0 mesh table, and the
  Player/NDrone2 DealWithObjHit water arms.

**The full suite did not fit.** The sandbox was reset this run (no repo, no toolchain, no game
files): clone + rustup + ISO + alsa-prefix setup consumed ~45 min. What remains for the next
message, per lesson 10 (code touched: world_fx.rs tick/attach, drone.rs footstep block,
scene.rs footstep block): every `walkreal_*_live` binary, `fix_flesh_impacts_live`,
`world_fx_live`, `lighting_live`, `fix_weapon_effects_live`, `impacts_live`,
`fix_foliage_bullets_live`, `mp_shot_effects_live`, `decal_*_live` — none were executed; both
changed crates' lib suites are green and the two lib test binaries compile clean.

## What's left

1. The run list above (walkreal set + the effect-adjacent live binaries).
2. A firing-through-the-real-path ripple pin: no walkable 0xD floor was confirmed under any
   level's start route this session, so the ripple's live pin is still the unit test plus the
   two wired call sites. A short walk test that crosses a 0xD/0xE floor and asserts a kind-1
   0x02000129 ring at the feet is the natural next pin.
3. The gas.rs kind-4 x/z jitter arm draws the raw [0,1) band − 0.025; the game adds
   `Rand_FRand(0x05)` per axis [M 0x1aacc8:92/96 → a 0.05 band]. Pre-existing stand-in, outside
   this task's scope, flagged by the new test's comment.

## Live checks for the main agent

1. Ripple material: with gdbwatch on `Env_WaterRipple` 0x17c118 (break on entry) while Bond
   wades in shallow water on any level with a pond (07000002 south-garden / pagoda pond):
   expect calls with `HITDATA+0x50 & 0x3f` = 0xd or 0xe on footstep frames. If the hits are
   0xe, the port's no_water footstep probe needs a follow-up so the player arm can fire there.
2. Flicker smoke: fieldprobe at any 0xe3 flicker placement over ~10 s — expect NO smoke puffs
   (the 0x1cc3d0 stub kills the branch), confirming the port's dead-gate model.

## Usage

not visible to me
